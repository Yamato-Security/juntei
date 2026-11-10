# LSA 비밀 정보 { #lsa-secrets }

## 개요 { #what-they-are }

LSA(Local Security Authority)는 비밀 정보를 `SECURITY` 하이브에 보관합니다. 여기에는 자동 로그온
암호, 서비스가 실행되는 계정의 암호, 컴퓨터 계정의 도메인 암호, DPAPI 시스템 키, 그리고 캐시된 도메인
로그온을 암호화하는 키가 포함됩니다. 또한 도메인 사용자가 도메인 컨트롤러 없이도 로그인할 수 있도록
캐시된 도메인 로그온도 보관합니다.

`parse-registry lsa-secrets`는 시스템이 어떤 비밀 정보를 보유하고 있으며 언제 설정되었는지, 그리고 캐시된
로그온을 몇 개 보관하고 사용하는지를 나열합니다. **이 명령은 이름과 시간만 읽습니다. 아무것도 복호화하지
않으며, 어떤 비밀 정보의 바이트도 읽거나 쓰지 않습니다.** Juntei는 증거를 읽을 뿐 자격 증명을 추출하지
않습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 시스템이 이 이름의 비밀 정보를 보유하고 있었으며,
이 시간에 설정되었습니다. 서비스의 비밀 정보(`_SC_<service>`)는 LSA가 해당 서비스가 실행되는 계정의
암호를 저장하고 있음을 의미합니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| 이름 | 종류 | 설명 |
|---|---|---|
| `DefaultPassword` | `autologon` | Winlogon이 자동으로 로그인할 때 사용하는 암호입니다. |
| `_SC_<service>` | `service_account` | `<service>`가 실행되는 계정의 암호입니다. |
| `$MACHINE.ACC` | `machine_account` | 도메인에서 컴퓨터 계정의 암호입니다. |
| `DPAPI_SYSTEM` | `dpapi_system` | 시스템의 DPAPI 키입니다. |
| `NL$KM` | `cached_logon_key` | 캐시된 로그온을 암호화하는 키입니다. |
| `L$_SQSA_<SID>` | `security_questions` | 로컬 계정의 보안 질문입니다(Windows 10 이상). |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | 전화 접속 및 VPN 자격 증명입니다. |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | `LsaStorePrivateData`가 부여하는 접두사로 구분되는 기타 개인 데이터입니다. |

`SECURITY`를 트랜잭션 로그와 함께 수집하고, 호스트 이름을 알 수 있는 시스템의 `SYSTEM` 하이브도
수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **값이 아니라 이름과 시간입니다.** 값에 데이터가 있는지 여부는 크기로 판단합니다. 그 내용은 읽지
  않습니다.
- **`DefaultPassword`가 자동 로그온을 의미하지는 않습니다.** 이 비밀 정보는 한 번도 자동으로 로그인하지 않는
  시스템을 포함해 대부분의 시스템에 존재합니다. 자동 로그온이 구성되어 있는지는 `SOFTWARE`의
  `Microsoft\Windows NT\CurrentVersion\Winlogon`(`AutoAdminLogon`, `DefaultUserName`)에서 알 수
  있습니다. 최근의 `CurrentSetUTC`는 설명이 필요한 변경입니다.
- **서비스 비밀 정보:** `_SC_<service>`는 서비스가 암호가 있는 계정으로 실행되도록 설정되었을 때
  나타납니다. 이를 `parse-registry services`(`ObjectName`)와 비교하고, 최근에 설치되었거나 알 수 없는
  서비스를 찾으십시오. 서비스가 사라진 뒤에도 비밀 정보는 남아 있을 수 있습니다.
- **시간:** `CurrentSetUTC`와 `OldSetUTC`는 Windows가 현재 값과 이전 값을 설정한 시점입니다.
  `$MACHINE.ACC` 암호는 기본적으로 30일마다 변경되므로 그 시간도 바뀝니다. `KeyLastWrittenUTC`와
  `CurrVal`/`OldVal` 키 시간은 비밀 정보가 마지막으로 변경된 시점에 관한 정보입니다.
- **캐시된 로그온:** 레코드에 사용자 이름이 있으면 해당 슬롯은 사용 중입니다. 레코드의 헤더는 암호화되어
  있지 않으며, 사용자 이름 길이만 읽습니다. `CachedLogonsKept`는 `NL$Control`의 개수이며,
  `CachedLogonsCount`(기본값 10)를 따릅니다. 사용 중인 슬롯은 도메인 사용자가 대화형으로 로그인했음을
  의미하며, 그 자격 증명은 하이브에서 오프라인으로 크래킹될 수 있습니다. 사용자 이름은 암호화되어 있으며
  디코딩하지 않습니다. 완전한 헤더와 일관된 사용자 이름 길이를 갖춘 검증된 바이너리 레코드만 사용 중으로
  집계됩니다. `Problems`가 손상된 슬롯을 보고하는 경우 사용 중 개수는 검증된 슬롯만 포함하므로, 손상된
  슬롯을 비어 있는 것으로 해석하지 마십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

요약에는 하이브별로 각 비밀 정보와 그 종류 및 설정 시점이 나열됩니다. 이어서 LSA가 계정 암호를 저장하고
있는 서비스의 이름을 표시하고, 사용 중인 캐시된 로그온 슬롯 수를 집계합니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트(시스템의 `SYSTEM` 하이브에서 얻음)와 `SECURITY` 하이브입니다. |
| `Secret`, `Kind` | 비밀 정보의 이름과 그 종류(위 표 참조)입니다. 캐시된 로그온 레코드의 경우 `Cache`와 `cached_logons`입니다. |
| `ServiceName`, `UserSID` | `_SC_` 비밀 정보의 대상 서비스와, 이름에 포함된 사용자의 SID(`L$_SQSA_`, `RasDialParams`)입니다. |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime`과 `OupdTime`: 현재 값과 이전 값이 설정된 시점입니다. |
| `HasCurrentValue`, `HasOldValue` | `CurrVal`과 `OldVal`에 데이터가 있는지 여부입니다. |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | `CurrVal`, `OldVal` 및 비밀 정보 키의 마지막 쓰기 시간입니다. |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | 캐시된 로그온 레코드: 슬롯 수, 사용 중인 슬롯 수, 그리고 `NL$Control`의 개수입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 잘못된 형식이거나 범위를 벗어난 시간(시간 메타데이터만 표시됨), 손상된 캐시 헤더, 잘못된 캐시 값 유형, 읽을 수 없는 키입니다. |

## 조사 전략 { #investigation-strategy }

1. 알 수 없거나 최근에 설치된 서비스의 `_SC_` 비밀 정보를 찾아, `parse-registry services` 및 이벤트
   7045(서비스 설치)와 매칭하십시오.
2. `DefaultPassword`의 시간을 `Winlogon`의 자동 로그온 설정과 대조하십시오.
3. 도메인 구성원에서는 사용 중인 캐시된 로그온 슬롯과 `Cache`가 마지막으로 기록된 시점을 기록해 두십시오.
4. 비밀 정보의 시간을 타임라인과 비교하십시오. 침입 기간 중에 설정된 비밀 정보는 새 서비스 계정, 자동 로그온
   또는 컴퓨터 계정 변경을 나타낼 수 있습니다.

## 추가 정보 { #further-details }

- [Microsoft: 개인 데이터 개체](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: 캐시된 도메인 로그온 정보](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: 캐시된 자격 증명](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
