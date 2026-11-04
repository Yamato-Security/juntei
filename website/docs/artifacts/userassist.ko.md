# UserAssist { #userassist }

## UserAssist란 { #what-userassist-is }

Windows Explorer는 사용자가 Explorer 셸(시작 메뉴, 작업 표시줄, 바탕 화면, 폴더에서의 더블 클릭)을 통해
시작한 프로그램과 바로 가기의 횟수를 셉니다. 이 횟수는 사용자의 `NTUSER.DAT`에 보관됩니다. 각 항목에는
다음이 있습니다.

- 실행 횟수
- Windows 7 이상에서는 포커스를 받은 횟수와 시간(밀리초)
- 마지막 실행 시간

값 이름은 ROT13으로 인코딩되어 있습니다. 프로그램 이름은 보통 알려진 폴더(known folder)의 GUID로
시작합니다(`{1AC14E77-…}`은 `System32`).

UserAssist는 특정 사용자가 대화형으로 시작한 것을 보여 줍니다. 대부분의 다른 실행 증거는 이를 사용자와
연결하지 못합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 사용자가 이 프로그램 또는 바로 가기를 Explorer를
통해 이 횟수만큼 시작했으며, 마지막 시작은 이 시간이었습니다. 다른 방법(명령줄, 서비스, 예약된 작업, 다른
프로그램)으로 시작된 프로그램은 집계되지 않습니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | 보관 내용 |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | 프로그램(`executable`), Windows 7 이상 |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | 바로 가기(`shortcut`), Windows 7 이상 |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | 프로그램 및 바로 가기(`active_desktop`), Windows 2000부터 Vista까지 |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | 인터넷 도구 모음(`internet_toolbar`), Windows 2000부터 Vista까지 |

그 밖의 GUID는 `other`로 보고됩니다. 모든 프로필의 `NTUSER.DAT`를 트랜잭션 로그와 함께 수집하고, 사용자의
SID를 제공하는 시스템의 `SOFTWARE` 하이브도 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **새 프로필에는 초기 항목이 들어 있습니다.** Windows 7 이상에서 새 프로필은 기본 항목(시작하기, 계산기,
  그림판, 스티커 메모, …)으로 시작합니다. 이들의 실행 횟수는 14부터 감소하며, 모두 프로필이 생성된 무렵의
  같은 마지막 실행 시간을 공유합니다. 이는 사용자 활동이 아닙니다.
- **버전 3은 5부터 셉니다**(Windows 2000부터 Vista까지). 저장된 횟수가 5부터 시작하므로 여기서 5를 뺍니다.
  저장된 횟수가 5 미만이면 `Problems`에 보고됩니다.
- **경로는 알려진 폴더의 기본 경로를 사용합니다**(`%windir%\system32`, `%ProgramFiles%`, `%APPDATA%\…`).
  다른 곳으로 리디렉션된 폴더도 기본 경로로 표시됩니다. 앱 ID(`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`)와 제어판 항목(`::{…}`)에는 경로가 없습니다.
- **추적은 끌 수 있습니다.** `Explorer\Advanced`의 `Start_TrackProgs` = 0, `Policies\Explorer`의
  `NoInstrumentation` = 1, 또는 `UserAssist\Settings`의 `NoLog` = 1로 끌 수 있습니다. 요약에는 추적이 꺼진
  하이브가 나열됩니다. 활성 프로필에서 UserAssist가 비어 있거나 없다면 안티 포렌식을 의미할 수 있습니다.
- **`NoEncrypt` = 1**이면 Windows가 ROT13 없이 이름을 기록합니다. 이러한 이름은 잘못 디코딩되므로, 해당
  하이브의 모든 항목에 그 사실이 `Problems`에 표시됩니다(레코드가 stdout으로 출력될 때는 stderr에도 표시).
- **마지막 실행 시간과 횟수는 항목별로 보관되지만,** 키의 마지막 쓰기 시간(`KeyLastWrittenUTC`)은 어느
  항목이든 가장 최근에 업데이트된 시간일 뿐입니다.
- **실행 횟수가 0일 수 있습니다.** 포커스만 받은 항목이 그렇습니다. 포커스 횟수와 시간은 버전 5(Windows 7
  이상)에만 존재합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

요약에는 마지막으로 실행된 항목이 사용자와 함께 최신 순으로 나열되며, UserAssist가 꺼진 하이브도 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 폴더 이름, 시스템의 `ProfileList`에서 얻은 사용자 SID(시스템의 `SOFTWARE` 하이브가 있는 경우), 그리고 하이브입니다. |
| `GUID`, `Kind` | `{GUID}` 키와 그 보관 내용(`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`)입니다. |
| `NameRot13`, `Name` | 저장된 그대로의 값 이름과 디코딩된 이름입니다. |
| `Path` | 알려진 폴더의 GUID를 기본 경로로 바꾼, 경로 형태의 디코딩된 이름입니다. 앱 ID와 셸 항목의 경우 비어 있습니다. |
| `RunCount` | Explorer에서 시작된 횟수입니다(버전 3: 저장된 횟수에서 5를 뺀 값). |
| `FocusCount`, `FocusTimeMs` | 포커스를 받은 횟수와 그 시간(밀리초)입니다(버전 5). |
| `LastRunUTC` | 마지막으로 시작된 시간입니다. Windows가 0을 저장했거나 저장된 시간이 9999년을 넘는 경우 비어 있습니다(이 경우 `Problems`에 그 값이 표시되고 `DataRaw`에 바이트가 보존됨). |
| `Format`, `SessionID` | 데이터의 레이아웃(5: 72바이트, 3: 16바이트)과 마지막으로 기록된 세션 번호입니다. |
| `KeyLastWrittenUTC` | `Count` 키의 마지막 쓰기 시간으로, 해당 키의 항목 중 하나가 가장 최근에 업데이트된 시간입니다. |
| `DataRaw` | 디코딩되지 않는 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 알 수 없는 크기의 데이터, 키의 `Version`과 맞지 않는 크기, 5 미만인 버전 3 횟수, 9999년을 넘는 마지막 실행 시간, `NoEncrypt`입니다. |

## 조사 전략 { #investigation-strategy }

1. `LastRunUTC`로 정렬하고 초기 항목을 제외하십시오. 초기 항목은 프로필 생성 무렵의 같은 시간을 공유합니다.
2. 사용자가 쓰기 가능한 위치(`%USERPROFILE%`, `Downloads`, `Temp`, `ProgramData`), 이동식 드라이브 및
   네트워크 경로에 있는 프로그램과, 관리 및 원격 액세스 도구를 찾으십시오.
3. Prefetch(실행 시간을 포함한 실행), Amcache와 ShimCache(존재), Jump Lists와 LNK 파일(그것으로 연 것),
   그리고 프로그램 파일에 대한 `$MFT`와 상관 분석하십시오.
4. 추적이 꺼진 하이브와 UserAssist가 비어 있는 프로필에서 기록 삭제 여부를 확인하십시오.
5. 삭제된 UserAssist 값을 찾으려면 `parse-registry read --recover-deleted`를 사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: User Assist keys](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
