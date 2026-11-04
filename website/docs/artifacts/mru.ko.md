# MRU(최근 사용) 목록 (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## 목록이란 { #what-the-lists-are }

Windows와 프로그램은 사용자가 마지막으로 입력하거나 연 항목의 짧은 목록을 사용자의
`NTUSER.DAT`에 보관합니다.

- **RunMRU:** 실행 대화 상자(Win+R)에 입력한 명령입니다: `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`.
- **TypedPaths:** 탐색기 주소 표시줄에 입력한 경로입니다.
- **WordWheelQuery:** 탐색기 검색 상자에서 검색한 검색어입니다.
- **TypedURLs:** Internet Explorer 주소 표시줄에 입력한 주소이며, Windows 8부터는
  `TypedURLsTime`에 각 주소를 입력한 시간이 기록됩니다.
- **Paint 및 WordPad:** 각 프로그램이 마지막으로 연 파일(`Recent File List`)입니다.
- **ACMru (Windows XP):** 검색 도우미(Search Assistant)의 검색어이며, 검색 종류별로 기록됩니다.

<span class="evidence-strength">가장 타당한 결론:</span> 사용자가 이 명령, 경로, 검색어 또는
주소를 입력했거나, 프로그램이 이 파일을 열었습니다. 각 목록에서 시간이 확인되는 것은 가장 최근
항목뿐이며, 이는 키의 마지막 쓰기 시간으로 결정됩니다. 단, 입력한 주소는 `TypedURLsTime`으로 시간이 확인됩니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

`ACMru`의 하위 키는 검색 종류를 나타냅니다: `5603` 파일 및 폴더 이름, `5604` 파일 내 단어 또는 구문,
`5647` 컴퓨터 및 사용자, `5001` 인터넷.

모든 프로필에 대해 `NTUSER.DAT`를 트랜잭션 로그와 함께 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **목록당 시간은 하나뿐입니다.** 키의 마지막 쓰기 시간은 목록이 바뀔 때 변경되므로, 가장 최근
  항목의 시간만 알려 줍니다(`IsMostRecent`, `LastUsedUTC`). 나머지 항목은 순서만 있고 시간은 없습니다.
  `TypedURLsTime`은 입력한 각 주소의 시간을 기록합니다(`TypedTimeUTC`).
- **입력한 것이지 실행한 것은 아닙니다.** RunMRU는 실행 대화 상자에 입력된 내용을 성공적으로 실행되었는지와
  관계없이 기록합니다. Windows는 각 명령 끝에 `\1`을 붙여 저장하며, Juntei는 이를 제거합니다. Prefetch,
  BAM, UserAssist 및 프로세스 생성 이벤트와 상관 분석하십시오.
- **목록은 짧습니다.** 각 목록은 가장 최근 항목만 유지하며, 개인 정보 설정과 정리 도구가 목록을 지웁니다.
  삭제된 값은 `parse-registry read --recover-deleted`를 사용하십시오.
- **검색어는 입력한 내용입니다.** WordWheelQuery와 ACMru는 찾은 결과가 아니라 입력한 텍스트를 보관합니다.
- **순서는 목록에서 결정됩니다.** RunMRU의 `MRUList`와 WordWheelQuery의 `MRUListEx`가 순서를 나타내며,
  나머지 목록은 값의 번호가 순서를 나타냅니다(`url1`, `File1`, `000`이 가장 최근 항목입니다).

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

요약에는 목록별 항목과 시간이 확인되는 항목(입력한 주소 및 각 목록의 가장 최근 항목)이 최신순으로
표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 소유자(경로 또는 셸 폴더에서 확인), 시스템의 `ProfileList`에서 가져온 SID, 그리고 하이브입니다. |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` 또는 `ACMru`입니다. |
| `Subkey` | `ACMru`의 하위 키로, 검색 종류(`5603`, `5604`, `5647`, `5001`)를 나타냅니다. |
| `MRU-Position`, `IsMostRecent` | 목록 내 위치(0이 가장 최근)입니다. 목록이 해당 값을 지정하지 않으면 비어 있습니다. |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | 명령(`\1` 제외), 경로, 검색어, 주소 또는 파일입니다. |
| `TypedTimeUTC` | `TypedURLsTime`에서 가져온, 주소를 입력한 시간입니다. |
| `LastUsedUTC` | 키의 마지막 쓰기 시간으로, 가장 최근 항목에만 기록됩니다. |
| `KeyPath`, `KeyLastWrittenUTC` | 키와 그 마지막 쓰기 시간입니다. 모든 행의 시간이 아니라 가장 최근 항목의 시간입니다. |
| `DataRaw` | 디코딩되지 않는 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 목록에는 지정되어 있으나 키에 없는 값, 목록에 지정되지 않은 값, 이름이 잘못된 값, 잘못된 형식, 잘린 검색어, FILETIME이 아닌 시간입니다. |

## 조사 전략 { #investigation-strategy }

1. RunMRU에서 사용자가 입력한 명령, 스크립트, 네트워크 공유 및 관리 도구를 확인하십시오.
2. TypedPaths와 WordWheelQuery에서 사용자가 찾으려 한 폴더와 파일을 확인하십시오:
   자격 증명, 내보내기 파일, 아카이브, 이동식 및 네트워크 위치 등.
3. `TypedTimeUTC`를 사용하여 입력한 주소를 타임라인에 배치하십시오.
4. Prefetch, BAM, UserAssist, ShellBags, LNK 파일, Jump Lists 및 브라우저 기록과 상관 분석하십시오.

## 추가 정보 { #further-details }

- [libyal winreg-kb: 최근 사용(MRU) 키](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
