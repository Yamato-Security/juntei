# MuiCache { #muicache }

## MuiCache란 { #what-muicache-is }

사용자가 프로그램을 시작하면 Windows는 셸에 표시할 이름(`FileDescription`, 예를 들어 `cmd.exe`의 경우
"Windows Command Processor")을 프로그램의 버전 리소스에서 읽어 사용자별로 `MuiCache`에 캐시합니다.
Windows 8 이상에서는 회사(`CompanyName`)도 캐시합니다. 캐시의 키는 프로그램의 전체 경로입니다.

- **Windows 8 이상:** 프로그램당 두 개의 값, `<path>.FriendlyAppName`과
  `<path>.AppCompany`.
- **Windows Vista 및 7:** 프로그램당 하나의 값, 이름(friendly name)을 담은 `<path>`.
- **Windows XP:** 같은 형식이며, `NTUSER.DAT`에 있습니다.

MuiCache는 사용자의 셸 세션에서 프로그램 실행에 관한 단서를 제공할 수 있습니다. 여기에는 이동식 드라이브,
네트워크 공유, 더 이상 존재하지 않는 폴더에 있던 프로그램도 포함되며, 파일 자체가 제공한 이름과 회사도
함께 기록됩니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 셸 프로필이 파일의 경로, 설명 및 회사를
캐시했습니다. 다른 실행 아티팩트와 상관 분석하십시오. 프로필은 프로세스가 실행된 계정을 식별하지
않습니다. MuiCache에는 프로그램별 시간도 실행 횟수도 없습니다.

## 위치 { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID`(캐시된 문자열의 언어)는 프로그램이 아니므로 제외됩니다. `@<file>,-<number>`
(`@shell32.dll,-22019`) 형식의 이름을 가진 값은 셸이 자체 메뉴와 파일 형식을 위해 캐시한 리소스
문자열이며 프로그램이 아닙니다. 이들은 제외되며 요약에서 개수가 집계됩니다. Windows Vista 이상에서는
`NTUSER.DAT`에 MuiCache가 없습니다.

모든 프로필의 `UsrClass.dat`(및 `NTUSER.DAT`)를 트랜잭션 로그와 함께 수집하고, 호스트 및 사용자 이름
확인을 위해 시스템의 `SYSTEM` 및 `SOFTWARE` 하이브도 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **프로필 소유자가 반드시 프로세스 계정인 것은 아닙니다.** 다른 계정으로 시작된 프로그램도 이 사용자의
  세션에 창을 표시하고 이 사용자의 캐시에 메타데이터를 남길 수 있습니다. `User`와 `UserSid`는 하이브의
  소유자를 식별하며, 프로세스 보안 컨텍스트를 식별하지 않습니다.
- **프로그램별 시간이 없습니다.** `KeyLastWrittenUTC`는 키의 마지막 쓰기 시간으로, 키에 속한 프로그램 중
  하나가 가장 최근에 변경된 시점이며 특정 프로그램이 실행된 시점이 아닙니다. 시간 정보는 Prefetch, BAM,
  UserAssist 및 이벤트 로그에서 얻을 수 있습니다.
- **사용자가 직접 시작한 프로그램만 있는 것은 아닙니다.** 셸 자체가 로드한 DLL이나 프로그램(`Open with`,
  셸 확장, 설치 프로그램의 하위 프로세스 등)이 `explorerframe.dll`이나 `shell32.dll`처럼 나타날 수
  있습니다.
- **이름은 파일에서 가져옵니다.** `FriendlyAppName`과 `AppCompany`는 실행 당시 파일의 버전 리소스에 기록된
  값입니다. 맬웨어는 어떤 설명과 회사든 주장할 수 있습니다. 이름이 바뀐 도구는 원래 설명을 유지하므로 정체가
  드러날 수 있습니다(이름은 `svchost.exe`이지만 표시 이름이 원격 액세스 도구인 파일 등).
- **경로당 하나의 엔트리.** 같은 프로그램을 다시 실행해도 엔트리가 추가되지 않지만, 다른 경로에 있는 사본은
  추가됩니다. 파일이 삭제된 후에도 프로그램의 엔트리는 남아 있을 수 있습니다.
- **하나의 경로에 두 가지 형식.** 하나의 경로에 두 형식의 표시 이름이 모두 있는 경우(업그레이드된 프로필)
  각각 별도의 레코드가 생성되므로 어느 값도 손실되지 않습니다.
- **손상된 값.** 문자열이 아니거나, 유효한 UTF-16이 아니거나, 전체를 읽을 수 없는 값은 바이트
  (`FriendlyAppNameRaw`, `AppCompanyRaw`)가 보존되고 `Problems`에 보고됩니다. 유효하지 않은 UTF-16은
  표시되는 텍스트에 대체 문자를 만들어 낼 수도 있으므로, 원래 바이트는 원시 열을 사용하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

요약에는 Windows 및 Program Files 폴더 밖에 있는 프로그램(사용자 폴더, 다른 드라이브, 네트워크 공유)이
사용자와 함께 나열되며, 제외된 리소스 문자열의 개수도 표시됩니다. 이름이 다른 Program Files 폴더(다른
언어의 Windows)도 나열됩니다. 상대 경로와 환경 변수 폴더를 벗어나는 경로는 계속 나열됩니다. 일반적인
드라이브 경로는 검사자의 파일 시스템에 접근하지 않고 `.` 및 `..` 구성 요소를 해석한 후 확인됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 폴더 이름, 사용자의 SID(`UsrClass.dat`의 루트 키 이름에서, 그렇지 않으면 시스템의 `ProfileList`에서 얻음), 그리고 하이브입니다. |
| `HiveType` | `UsrClass.dat` 또는 `NTUSER.DAT`입니다. |
| `Path` | 프로그램의 경로로, 값 이름에서 `.FriendlyAppName` 또는 `.AppCompany`를 제외한 것입니다. |
| `FriendlyAppName` | `<path>.FriendlyAppName` 또는 `<path>`에서 얻은 프로그램 설명입니다. |
| `AppCompany` | `<path>.AppCompany`에서 얻은 프로그램의 회사입니다(Windows 8 이상). |
| `KeyPath`, `KeyLastWrittenUTC` | MuiCache 키와 그 마지막 쓰기 시간, 즉 키에 속한 프로그램 중 하나가 가장 최근에 변경된 시점입니다. |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | 깔끔하게 디코딩되는 텍스트가 아닌 경우의 값 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. 로그가 프로그램의 값 중 하나를 기록한 경우 `replayed`입니다. |
| `Problems` | 문자열이 아니거나, 유효한 UTF-16이 아니거나, 읽을 수 없는 값입니다. |

## 조사 전략 { #investigation-strategy }

1. 사용자가 쓰기 가능한 위치(`\Users\`, `Downloads`, `Temp`, `ProgramData`), 다른 드라이브 및 네트워크
   공유에 있는 프로그램과 관리, 원격 액세스 및 압축 도구를 살펴보십시오.
2. 파일 이름을 표시 이름 및 회사와 비교하십시오. 불일치는 이름이 바뀐 도구나 위장 파일을 드러낼 수
   있습니다.
3. Prefetch, BAM 및 UserAssist(실행 시점), Amcache 및 ShimCache(파일의 해시와 존재), 그리고
   `$MFT`(파일이 아직 존재하는지 여부)와 상관 분석하십시오.
4. 삭제된 MuiCache 값은 `UsrClass.dat`에 `parse-registry read --recover-deleted`를 사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: VERSIONINFO 리소스](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: MuiCache 실험, GUI 세션 및 프로세스 계정의 한계](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
