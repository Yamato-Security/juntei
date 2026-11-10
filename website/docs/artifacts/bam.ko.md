# BAM / DAM { #bam-and-dam }

## BAM과 DAM이란 { #what-bam-and-dam-are }

Background Activity Moderator(BAM)와 Desktop Activity Moderator(DAM)는 애플리케이션이 백그라운드에서
수행할 수 있는 작업을 관리하는 Windows 서비스입니다(Windows 10 1709 이상). 이 서비스들은 사용자별로
각 프로그램이 마지막으로 실행된 시간을 `SYSTEM` 하이브에 보관합니다.

- 사용자당 하나의 키가 있으며, 키 이름은 사용자의 SID입니다.
- 프로그램당 하나의 값이 있으며, 값 이름은 장치상의 프로그램 경로
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) 또는 패키지 앱의 ID
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`)입니다.
- 값 데이터의 처음 8바이트는 마지막 실행 시간(FILETIME)입니다.

BAM은 사용자 자신의 하이브 없이도 프로그램을 사용자 및 시간과 연결하며, 이동식 장치와 네트워크
장치에서 실행된 프로그램도 나열합니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 프로그램의 프로세스가 이 사용자의 세션에서
실행되었으며, 마지막 실행 시간은 이 시간입니다. BAM은 실행 횟수를 세지 않으며, 프로그램이 삭제된 후에도
항목이 남아 있을 수 있습니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

각 사용자의 키에는 `Version`과 `SequenceNumber` 값도 있지만, 이는 프로그램이 아니므로 제외됩니다.
1709 또는 1803에서 업그레이드된 시스템에는 두 레이아웃이 모두 존재할 수 있습니다.

`SYSTEM`을 트랜잭션 로그와 함께 수집하고, 사용자 이름이 들어 있는 시스템의 `SAM`과 `SOFTWARE`도
수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **프로그램 및 사용자당 하나의 시간.** 가장 최근 실행만 보관됩니다. 이전 실행은 Prefetch, Amcache
  및 이벤트 로그에서 확인할 수 있습니다.
- **최근 활동.** 한동안 실행되지 않은 프로그램의 항목은 Windows 시작 시 제거되는 것으로 알려져
  있으므로, BAM은 주로 최근 활동을 보여 줍니다. BAM에 없는 프로그램도 오래전에 실행되었을 수 있습니다.
- **드라이브 문자가 아닌 장치 경로.** `\Device\HarddiskVolume3`은 Windows가 세 번째로 번호를 매긴
  볼륨이며 `C:`가 아닙니다. `DevicePath`와 `Path`는 장치와 그 장치상의 경로를 분리합니다. 드라이브
  문자를 기재하기 전에 `MountedDevices`, `$MFT` 또는 기타 아티팩트로 볼륨을 대조하십시오.
- **모든 프로그램이 아닙니다.** BAM과 DAM은 사용자 세션의 프로그램을 추적하므로 서비스는 대부분
  나타나지 않습니다. `S-1-5-18` 항목은 일반적으로 설치 및 로그인 화면의 프로세스
  (`FirstLogonAnim.exe`)이고, `S-1-5-90-0-<n>` 항목은 Desktop Window Manager(`DWM-<n>`)입니다.
- **제어 집합.** 기본적으로 Windows가 마지막으로 사용한 제어 집합을 읽습니다. `--all-control-sets`는
  모든 제어 집합을 읽습니다. 이전 제어 집합에는 현재 제어 집합에 더 이상 없는 항목이 남아 있을 수 있습니다.
- **사용자 이름.** 로컬 계정의 이름은 `SAM`(컴퓨터의 SID와 계정의 RID)에서, 도메인 계정의 이름은
  `ProfileList`의 프로필 폴더에서 얻습니다. `UserSource`는 그 출처(`sam`, `profile_list`,
  `well_known`)를 나타냅니다. 이러한 하이브가 없으면 SID만 제공됩니다. `SAM`에 더 이상 없는 RID
  (삭제된 계정)는 이름 없이 남습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

요약에는 각 사용자가 SID, 이름 및 항목 수와 함께 나열되고, 마지막으로 실행된 프로그램이 최신 순으로
표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트와 `SYSTEM` 하이브입니다. |
| `ControlSet`, `IsCurrentControlSet` | 항목이 속한 제어 집합과, 그것이 Windows가 마지막으로 사용한 제어 집합인지 여부입니다. |
| `Moderator` | `bam` 또는 `dam`입니다. |
| `UserSID`, `User`, `UserSource` | 키 이름인 SID, 사용자 이름, 그리고 이름의 출처(`sam`, `profile_list`, `well_known`)입니다. |
| `Program` | Windows가 기록한 그대로의 값 이름으로, 장치 경로 또는 패키지 앱의 ID입니다. |
| `DevicePath`, `Path` | 장치 경로인 경우 장치(`\Device\HarddiskVolume3`)와 그 장치상의 경로(`\Windows\System32\cmd.exe`)이며, 패키지 앱인 경우 비어 있습니다. |
| `LastRunUTC` | 프로그램이 마지막으로 실행된 시간입니다(100ns 단위). |
| `KeyPath`, `KeyLastWrittenUTC` | 사용자의 키와 그 마지막 쓰기 시간으로, 해당 키의 항목 중 하나가 가장 최근에 업데이트된 시간입니다. |
| `DataRaw` | 값에 시간이 들어 있지 않은 경우(`REG_BINARY`가 아니거나 8바이트 미만) 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 시간이 들어 있지 않은 데이터와 읽을 수 없는 값입니다. |

## 조사 전략 { #investigation-strategy }

1. `LastRunUTC`로 정렬하고, 사용자가 쓰기 가능한 위치(`\Users\`, `Downloads`, `Temp`,
   `ProgramData`)에 있는 프로그램, 시스템 볼륨 이외의 장치에 있는 프로그램, 관리 및 원격 액세스 도구를
   살펴보십시오.
2. 어떤 사용자가 실행했는지, 그리고 현재 프로필과 일치하지 않는 계정이 있는지 확인하십시오.
3. Prefetch(실행 횟수 및 이전 실행), UserAssist(Explorer를 통한 시작), Amcache와 ShimCache(존재),
   `$MFT`(프로그램 파일) 및 이벤트 로그와 상관 분석하십시오.
4. 삭제된 BAM 값을 찾으려면 `--all-control-sets`와 `parse-registry read --recover-deleted`를
   사용하십시오.

## 추가 정보 { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: 보안 식별자(잘 알려진 SID)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

범위를 벗어난 마지막 실행 FILETIME은 `LastRunUTC`를 비워 두고 원래 값을 `DataRaw`에 보존하며,
`Problems`에 설명을 남깁니다. 최근 활동 요약은 각 호스트를 식별하고 서로 다른 하이브/사용자 관찰 결과를
구별하여 유지합니다.
