# 시스템 정보 { #system-information }

## 개요 { #what-it-is }

시스템의 네 가지 머신 하이브는 분석가가 타임라인을 읽기 전에 알아야 할 정보를 알려 줍니다.

- 시스템의 이름(NetBIOS, DNS 호스트 이름, DNS 도메인)과 소속된 도메인 또는 작업 그룹
- Windows 버전, 빌드 및 업데이트 리비전, 그리고 Windows가 설치되었거나 마지막으로 업그레이드된 시점
- 시스템이 기록한 로컬 시간을 알려 주는 표준 시간대
- 마지막으로 정상 종료된 시점
- 프로세서 아키텍처와 코드 페이지
- 마지막으로 로그인한 사용자와, 로그인한 적이 있는 모든 사용자의 프로필
- SID: 컴퓨터의 SID(로컬 계정은 이 SID에 RID를 더한 것)와 도메인의 SID
- 업그레이드되기 이전의 Windows 버전

`parse-registry system-info`는 이 모든 정보를 담은 레코드를 시스템당 하나씩 출력합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 하이브가 수집된(또는 마지막으로 기록된) 시점의
구성입니다. 키 시간은 그 일부가 마지막으로 변경된 시점을 알려 줍니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

네 하이브를 트랜잭션 로그와 함께 수집하십시오. 하나만 있어도 그 하이브가 가진 정보는 얻을 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **레지스트리 시간은 UTC이며, 표준 시간대는 그 밖의 모든 것을 위한 것입니다.** 키 시간, FILETIME, 대부분의
  이벤트 시간은 UTC입니다. 텍스트 로그, 일부 애플리케이션 로그, 이름이나 문자열에 기록된 날짜
  (`Source OS (Updated on …)` 등)는 로컬 시간일 수 있습니다. `Bias`는 로컬 표준시에서 UTC까지의 분
  (`UTC = local + Bias`)입니다. `UtcOffset`과 `DaylightUtcOffset`은 두 오프셋을 제공하며,
  `TimeZoneDescription`은 일광 절약 시간이 언제 시작되고 끝나는지 알려 줍니다.
- **표준 시간대는 변경될 수 있습니다.** `TimeZoneLastWrittenUTC`는 키의 마지막 쓰기 시간입니다. Windows는
  `ActiveTimeBias`(그 시점에 적용 중인 오프셋)를 갱신할 때 스스로 키를 다시 기록하므로, 최근 시간이라고 해서
  누군가가 표준 시간대를 변경했다는 증거가 되지는 않습니다. 이벤트 로그와 비교하십시오. Security 4616(시스템
  시간 변경)과 System 로그의 `Microsoft-Windows-Kernel-General` 이벤트입니다.
- **`StandardStart`와 `DaylightStart`는 `SYSTEMTIME`이 아니라 커널의 `TIME_FIELDS`입니다.** 요일은 세 번째
  필드(`SOFTWARE`에 있는 표준 시간대의 `TZI` 값처럼)가 아니라 마지막 필드입니다. 미국 표준 시간대에서는 요일이
  일요일(0)이므로 두 레이아웃이 동일하게 읽힙니다. Juntei는 이를 `TIME_FIELDS`로 디코딩합니다. 공개된 이스터섬
  시스템(첫 번째 토요일 22:00)에서 그 차이를 볼 수 있습니다. 월이 0이면 일광 절약 시간이 없다는 뜻입니다.
- **Windows 11은 "Windows 10"이라고 표시합니다.** Windows 11에서도 `ProductName`은 `Windows 10 …`을
  유지합니다. 빌드 22000 이상은 Windows 11이며, 요약에는 그렇게 표시됩니다. `CurrentVersion`은 Windows 8.1,
  10, 11에서 `6.3`이며, `Version`은 `CurrentMajorVersionNumber`(10)가 있으면 이를 사용합니다.
- **`InstallDate`는 기능 업그레이드와 함께 바뀝니다.** 새 릴리스로의 전체 업그레이드(예: 1809에서 21H2로)는
  이를 재설정합니다. 각 업그레이드가 대체한 릴리스와 그 고유한 `InstallDate`는 `PreviousVersions`
  (`Setup\Source OS`)에 있습니다. 그중 가장 이른 `InstallDate`가 원래 설치 시점에 가장 가깝습니다.
  `Source OS` 키 이름에 있는 날짜는 표준 시간대를 알려 주지 않습니다.
- **`ShutdownTime`은 마지막 정상 종료입니다.** 충돌이나 전원 손실은 이를 갱신하지 않습니다. 이벤트 로그
  (6006, 6008, 1074) 및 하이브에 대한 마지막 쓰기 시간과 비교하십시오.
- **`LastLoggedOnUser`는 로그인 화면의 마지막 사용자이며,** 모든 로그온이 아닙니다. 원격 데스크톱, `runas`,
  서비스, 예약된 작업이 모두 이를 설정하는 것은 아닙니다. `LogonUiLastWrittenUTC`는 그것이 마지막으로 설정된
  대략적인 시점입니다.
- **프로필:** `ProfileList`는 시스템에 프로필을 가졌던 모든 사용자(및 서비스)를 나열하며, 삭제된 계정도
  프로필이 제거될 때까지 포함됩니다. Windows 10 이상은 `LocalProfileLoadTime`과 `LocalProfileUnloadTime`을
  기록하며, 이는 대략 마지막 로그인과 로그아웃입니다.
- **SID:** 로컬 계정은 `MachineSid-<RID>`, 도메인 계정은 `DomainSid-<RID>`입니다. `MachineSid`는 `SAM`에서
  (없으면 `SECURITY`의 `PolAcDmS`에서) 가져옵니다. 두 값은 일치해야 하며, 일치하지 않으면 서로 다른 시스템의
  하이브인 경우처럼 `Problems`에 그 사실이 표시됩니다. 작업 그룹에서는 `DomainSid`가 비어 있습니다.
- **디렉터리 하나에 시스템 하나.** `-d`를 사용하면 한 디렉터리에 있는 하이브는 하나의 시스템의 것으로
  간주됩니다. `RegBack` 사본과 같은 종류의 두 번째 하이브(`SYSTEM.old`)는 별도의 레코드가 됩니다. `-f`를
  사용하면 지정한 하이브들은 하나의 시스템의 것으로 간주되며, 나머지 하이브는 그 옆에서 찾습니다.
- **제어 집합:** Windows가 마지막으로 사용한 제어 집합만 읽습니다. 다른 제어 집합은 `parse-registry read`로
  확인할 수 있습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

요약에는 각 시스템에 대해 Windows 버전, 설치 시점, 문장으로 표현한 표준 시간대, 키가 마지막으로 기록될 때
적용 중이던 오프셋, 마지막 종료, 도메인 또는 작업 그룹, 마지막으로 로그인한 사용자, SID, 프로필 수, 그리고
업그레이드 이전 버전이 표시됩니다. `auto-parse`도 발견한 시스템을 나열할 때 각 시스템의 표준 시간대를
출력합니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | 호스트와 읽은 하이브(`SourcePath`는 `SYSTEM` 하이브, 없으면 처음 읽은 하이브). |
| `CompName`, `Hostname`, `Domain` | NetBIOS 이름(`SYSTEM`, 없으면 `SECURITY`의 `PolAcDmN`), DNS 호스트 이름 및 DNS 도메인(`Tcpip\Parameters`). |
| `PrimaryDomain`, `DNS-Domain` | 컴퓨터가 가입된 도메인 또는 작업 그룹(`PolPrDmN`), 그리고 도메인의 DNS 이름(`PolDnDDN`). |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: 기록된 그대로의 제품 이름, `EditionID`, `DisplayVersion`(없으면 `ReleaseId`), 주 버전과 부 버전(없으면 `CurrentVersion`), 빌드, 업데이트 빌드 리비전, `CSDVersion`, 그리고 `Client`, `Server` 또는 `Server Core`. |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate`(초 단위)와 `InstallTime`(100 ns 단위, Windows 10 이상): 설치 또는 마지막 기능 업그레이드. |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | 기록된 그대로. |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName`(없으면 `StandardName`), 그리고 오프셋과 일광 절약 시간 날짜를 포함하여 문장으로 표현한 표준 시간대. |
| `UTC-Offset`, `DaylightUTC-Offset` | 로컬 표준시의 UTC 오프셋과 일광 절약 시간의 오프셋(일광 절약 시간이 없으면 비어 있음). |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias`(키가 기록될 때 적용 중), `StandardBias`, `DaylightBias`. |
| `StandardName`, `DaylightName` | 기록된 그대로이며, 흔히 리소스 참조(`@tzres.dll,-112`)입니다. |
| `StandardStart`, `DaylightStart` | 표준시와 일광 절약 시간이 시작되는 시점을 문장으로 표현한 것(`first Sunday of November, 02:00`). |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | 동적 일광 절약 시간이 꺼져 있음. 하드웨어 시계가 UTC를 유지함. |
| `TimeZoneLastWrittenUTC` | 표준 시간대 키의 마지막 쓰기 시간. |
| `LastShutdownUTC` | `ShutdownTime`: 마지막 정상 종료. |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | 읽은 제어 집합, `PROCESSOR_ARCHITECTURE`, `ACP` 및 `OEMCP`. |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | 로그인 화면의 마지막 사용자와 그 키의 마지막 쓰기 시간. |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | 컴퓨터의 SID, 도메인의 SID(`PolPrDmS`), 도메인 내 컴퓨터 계정의 SID(`PolMachineAccountS`). |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | 컴퓨터 이름이 마지막으로 지정된 시점, 도메인 또는 작업 그룹에 마지막으로 가입한 시점, 마지막으로 업데이트된 시점(`CurrentVersion`의 키)의 대략적인 시간. |
| `PreviousVersions` | 각 `Source OS` 키: 버전, 설치 날짜, 키 이름에 있는 날짜(JSON: 각각 하나의 개체). |
| `Profiles` | 각 프로필: SID, 폴더, 키 시간, 마지막 로드 및 언로드(JSON: 각각 하나의 개체). |
| `TransactionLogStatus`, `LogsApplied` | 네 하이브 전체에 대한 레지스트리 복구 컨텍스트. |
| `Problems` | 잘못된 유형이나 크기의 값(바이트는 보존), 그리고 컴퓨터 SID가 서로 다른 `SAM`과 `SECURITY`. |

## 조사 전략 { #investigation-strategy }

1. 먼저 표준 시간대를 읽습니다. `UtcOffset`, 일광 절약 시간 날짜, 표준 시간대가 최근에 변경되었는지 여부를
   기록하고, 이를 사용하여 로컬 시간 로그를 변환합니다.
2. 버전, 빌드, 설치 날짜를 기록합니다. `PreviousVersions`에서 업그레이드를 확인하고, 설치 날짜를 가장 오래된
   파일 시스템 및 이벤트 로그 시간과 비교합니다.
3. `LastShutdownUTC`를 기록합니다. 그 이후에 새로운 종료 시간 없이 활동이 있다면 충돌, 전원 손실 또는 라이브
   수집을 시사합니다.
4. `MachineSid`와 `DomainSid`를 사용하여 다른 곳(BAM, `ProfileList`, 이벤트 로그)의 SID에서 로컬 계정과
   도메인 계정을 구별하고, 프로필의 로드 및 언로드 시간으로 마지막 로그인을 확인합니다.
5. 컴퓨터 이름과 도메인 이름을 이벤트 로그와 상관 분석하고, 이름 변경과 도메인 가입은
   `CompNameLastWrittenUTC` 및 `PrimaryDomainLastWrittenUTC`와 상관 분석합니다.

## 추가 정보 { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: 보안 식별자](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: 표준 시간대](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: Windows 제품 정보](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
