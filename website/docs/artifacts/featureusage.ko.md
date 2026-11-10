# FeatureUsage, RecentApps 및 작업 표시줄 고정 항목 { #featureusage-recentapps-and-taskbar-pins }

## 개요 { #what-they-are }

탐색기(Explorer)는 사용자가 애플리케이션을 어떻게 사용하는지에 관한 여러 기록을 해당 사용자의
`NTUSER.DAT`에 보관합니다.

- **FeatureUsage**(Windows 10 이상): 애플리케이션별, 사용 종류별 카운터입니다.
  - `AppSwitched`: 사용자가 해당 애플리케이션의 창으로 전환한 횟수(작업 표시줄 클릭,
    Alt+Tab)입니다.
  - `AppLaunch`: 작업 표시줄(고정된 항목)에서 시작된 횟수입니다.
  - `ShowJumpView`: 점프 목록을 연 횟수(작업 표시줄에서 마우스 오른쪽 클릭)입니다.
  - `AppBadgeUpdated`: 작업 표시줄 배지(읽지 않은 항목 수)가 변경된 횟수입니다.
  - `TrayButtonClicked`: 각 작업 표시줄 단추(`StartButton`, `SearchButton`,
    `TaskViewButton`, `ClockButton`, …)를 클릭한 횟수입니다.

  애플리케이션은 애플리케이션 사용자 모델 ID(`Microsoft.Windows.Explorer`,
  `Microsoft.WindowsCalculator_8wekyb3d8bbwe!App`), 첫 부분이 알려진 폴더(known folder)의 GUID일
  수 있는 경로(`{1AC14E77-…}\cmd.exe`는 `%windir%\system32\cmd.exe`), 또는 탐색기가 애플리케이션과
  연결하지 못한 창의 경우 프로세스 ID(`*PID00001d74`)로 표시됩니다.
- **RecentApps**(Windows 10 1709까지): 사용자가 최근에 실행한 애플리케이션과 그 실행 횟수 및 마지막
  시간이며, 각 애플리케이션 아래에는 해당 애플리케이션이 연 파일(`RecentItems`)이 있습니다.
- **작업 표시줄 고정 항목**(`Taskband`): 작업 표시줄에 고정된 프로그램으로, 셸 항목(`Favorites`)과
  고정된 각 바로 가기의 링크(`FavoritesResolve`)로 기록됩니다. Windows 10 이상에서는 각 고정 항목에
  Windows가 고정했는지(`SystemPinned`) 사용자가 고정했는지(`UserPinned`)가 기록됩니다.
- **ProgramsCache**(`StartPage`, `StartPage2`; Windows XP부터 7까지): 시작 메뉴에 캐시된
  프로그램(`ProgramsCache`)이며, Windows 7에서는 시작 메뉴와 작업 표시줄에 고정된 프로그램
  (`ProgramsCacheSMP`, `ProgramsCacheTBP`)도 포함됩니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 사용자가 이 애플리케이션을 이 방식으로
이만큼 사용했거나(FeatureUsage), 이 시간에 실행했거나(RecentApps), 고정했습니다. 카운터 자체에는 시간이
없으며, 시점을 알려 주는 것은 키의 마지막 쓰기 시간과 RecentApps의 시간뿐입니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\FeatureUsage\<kind>     value: application, data: count
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}         AppId, AppPath, LaunchCount, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Search\RecentApps\{GUID}\RecentItems\{GUID}   DisplayName, Path, LastAccessedTime
Software\Microsoft\Windows\CurrentVersion\Explorer\Taskband                Favorites, FavoritesResolve
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage2              ProgramsCache, ProgramsCacheSMP, ProgramsCacheTBP
Software\Microsoft\Windows\CurrentVersion\Explorer\StartPage               ProgramsCache (Windows XP, Vista)
```

`FeatureUsage`의 `KeyCreationTime`은 집계를 시작한 시점이며, 요약에 표시됩니다. 모든 프로필의
`NTUSER.DAT`를 트랜잭션 로그와 함께 수집하고, 호스트 및 사용자 SID 확인을 위해 시스템의 `SYSTEM` 및
`SOFTWARE` 하이브도 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **횟수는 있지만 시간은 없습니다.** FeatureUsage 횟수는 얼마나 자주인지를 알려 줄 뿐 언제인지는
  알려 주지 않습니다. 종류별 키의 마지막 쓰기 시간(`KeyLastWrittenUTC`)은 해당 키의 횟수 중 하나가
  가장 최근에 변경된 시점입니다.
- **모든 실행이 기록되지는 않습니다.** `AppSwitched`는 시작이 아니라 창 전환을 집계합니다. 창 없이
  실행되었거나 사용자가 한 번도 전환하지 않은 애플리케이션은 집계되지 않습니다. `AppLaunch`는 작업
  표시줄에서의 시작만 집계합니다.
- **패키지 앱과 경로.** 애플리케이션 이름은 Windows가 창을 그룹화한 방식대로 지정됩니다. 패키지 앱의
  ID, 프로그램이 스스로 설정한 ID(`Chrome`, `VMware.Workstation.vmui`), 또는 경로입니다. `AppPath`는
  알려진 폴더의 GUID를 해당 폴더의 기본 경로로 바꾼 경로를 제공합니다.
- **기본 고정 항목.** 새 프로필에는 기본 고정 항목(파일 탐색기, Edge, Store)이 생성되며, Windows 10
  이상에서는 `SystemPinned`로 표시됩니다. `UserPinned` 고정 항목은 사용자의 선택입니다.
- **두 목록에 나타나는 고정 항목.** `Favorites`와 `FavoritesResolve`는 같은 고정 항목을 같은
  순서(`Index`)로 설명하지만, 별도의 레코드로 제공됩니다.
- **ProgramsCache 경로**는 상대 경로입니다. 폴더(`Programs\Accessories`) 다음에 그 안의
  바로 가기(`Notepad.lnk`)가 이어집니다.
- **손상된 값.** `REG_DWORD`가 아닌 횟수, `REG_QWORD`가 아닌 시간, 해석할 수 없는 고정 항목, 또는
  링크가 아닌 링크는 바이트(`DataRaw`)가 보존되고 `Problems`에 보고됩니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry feature-usage <INPUT> [OPTIONS]

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
juntei parse-registry feature-usage -d evidence/ -o featureusage.csv
juntei parse-registry feature-usage -f NTUSER.DAT -t jsonl \
  | jq 'select(.Source == "AppSwitched") | [.Count, .AppPath // .App]'
```

요약에는 원본별 레코드, 각 하이브의 FeatureUsage가 집계를 시작한 시점, 가장 많이 전환된 애플리케이션,
최근 앱 및 항목(최신순), 그리고 작업 표시줄 고정 항목이 나열됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 폴더 이름, 시스템의 `ProfileList`에서 얻은 사용자 SID, 그리고 하이브입니다. |
| `Source` | `AppSwitched`, `AppLaunch`, `ShowJumpView`, `AppBadgeUpdated`, `TrayButtonClicked`(FeatureUsage), `RecentApps`, `RecentItems`, `Taskband` 또는 `ProgramsCache`입니다. |
| `App` | FeatureUsage 값의 이름, 최근 앱의 `AppId`, 또는 고정 항목의 애플리케이션 사용자 모델 ID입니다. |
| `AppPath` | 경로로 표현한 `App`으로, 알려진 폴더의 GUID를 기본 경로로 바꾼 것입니다. ID인 경우 비어 있습니다. |
| `Count` | FeatureUsage 횟수 또는 최근 앱의 `LaunchCount`입니다. |
| `LastAccessedUTC` | 최근 앱 또는 항목의 `LastAccessedTime`입니다. |
| `ItemPath` | 최근 항목의 `Path`(또는 최근 앱의 `AppPath`), 고정 항목이나 캐시된 프로그램의 셸 항목 경로(`User Pinned\TaskBar\File Explorer.lnk`, `Applications\<app ID>`), `FavoritesResolve` 링크의 경로입니다. |
| `DisplayName` | 최근 항목의 `DisplayName`입니다. |
| `PinType` | `Favorites` 고정 항목의 `SystemPinned` 또는 `UserPinned`입니다. |
| `Index` | 고정 항목 또는 캐시된 프로그램의 목록 내 위치입니다. |
| `ValueName`, `KeyPath`, `KeyLastWrittenUTC` | 레코드의 출처인 값과 키, 그리고 키의 마지막 쓰기 시간입니다. |
| `DataRaw` | 디코딩되지 않는 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 잘못된 유형의 값, 해석할 수 없는 고정 항목이나 목록, 링크가 아닌 링크입니다. |

## 조사 전략 { #investigation-strategy }

1. `AppSwitched`를 `Count` 기준으로 정렬하여 사용자가 가장 많이 작업한 애플리케이션을 확인하고, 관리,
   원격 액세스, 압축 및 전송 도구와 사용자가 쓰기 가능한 위치(`\Users\`, `Downloads`, `Temp`)의 경로를
   찾으십시오.
2. 고정 항목을 확인하십시오. `UserPinned`로 고정된 도구나 바로 가기가 특이한 위치에 있는 고정 항목에
   주목하십시오.
3. RecentApps의 시간을 UserAssist, Prefetch, BAM 및 Jump Lists와 함께 사용하여 사용 시점을 확정하십시오.
4. `KeyCreationTime`을 프로필 생성 시점과 비교하십시오. 집계는 그때부터 시작됩니다.
5. 삭제된 카운터와 고정 항목은 `parse-registry read --recover-deleted`를 사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 애플리케이션 사용자 모델 ID](https://learn.microsoft.com/en-us/windows/win32/shell/appids)
- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: Programs cache](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Program-cache.html)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
