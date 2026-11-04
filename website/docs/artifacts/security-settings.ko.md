# 보안 설정 { #security-settings }

## 개요 { #what-they-are }

공격자는 탐지를 피하고 자격 증명에 접근하기 위해 Windows의 보안 설정을 변경합니다. Microsoft Defender를
끄거나 자신의 도구를 검사 대상에서 제외하고, Windows가 평문 암호를 보관하도록 만들고, 원격 데스크톱을
열고, 방화벽을 끄고, PowerShell 로깅을 무력화합니다. 레지스트리에는 다음과 같은 설정이 보관됩니다.

- **`SOFTWARE`:** Defender와 그 제외 항목 및 정책, UAC, 원격 데스크톱 및 방화벽 정책, PowerShell의 실행
  정책, 로깅 정책 및 AMSI 공급자, 시스템 복원, 이벤트 로그 채널.
- **`SYSTEM`**(제어 집합별): `WDigest`, LSA 보호, Credential Guard, Restricted Admin, NTLM, 원격
  데스크톱, 방화벽 프로필, SMBv1, 마지막 액세스 시간, 각 이벤트 로그의 크기.
- **`NTUSER.DAT`:** 사용자의 PowerShell 실행 정책 및 로깅 정책, 그리고 Windows Script Host의 AMSI.

`parse-registry security-settings`는 발견한 각 설정을 하나의 레코드로 보고합니다. 각 레코드에는 설정 값,
그 값의 의미, 그리고 해당 값이 **기본값을 약화시키는지** 여부가 포함됩니다. 즉, 보호 기능이나 증거 출처를
끄는지, 또는 해당 값이 없을 때 현재 Windows(10 및 11)가 동작하는 수준보다 낮추는지를 나타냅니다.

<span class="evidence-strength">가장 타당한 결론:</span> 하이브가 저장된 시점에 이 설정은 이 값을 가지고
있었으며, 키는 이 시간에 마지막으로 기록되었습니다. 약화된 설정은 단서이지 침해의 증명이 아닙니다.
관리자, 그룹 정책, 다른 보안 제품 및 이전 Windows 버전도 이러한 값 중 일부를 설정하며, 그런 경우가 흔할
때는 의미 설명에 그렇게 표시됩니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Defender[\Real-Time Protection|\Features|\Spynet]
  Microsoft\Windows Defender\Exclusions\Paths|Extensions|Processes|IpAddresses|TemporaryPaths
  Policies\Microsoft\Windows Defender\…
  Microsoft\Windows\CurrentVersion\Policies\System                 UAC
  Policies\Microsoft\Windows NT\Terminal Services                  RDP policy
  Policies\Microsoft\WindowsFirewall\<profile>                      firewall policy
  Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell             ExecutionPolicy
  Policies\Microsoft\Windows\PowerShell\ScriptBlockLogging|ModuleLogging|Transcription
  Microsoft\AMSI\Providers\{CLSID}
  Microsoft\Windows NT\CurrentVersion\SystemRestore                DisableSR
  Microsoft\Windows\CurrentVersion\WINEVT\Channels\<channel>       Enabled, MaxSize
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Control\SecurityProviders\WDigest                                UseLogonCredential
  Control\Lsa                                                      RunAsPPL, LsaCfgFlags, DisableRestrictedAdmin, LmCompatibilityLevel
  Control\Terminal Server[\WinStations\RDP-Tcp]                    fDenyTSConnections, PortNumber, UserAuthentication
  Services\SharedAccess\Parameters\FirewallPolicy\<profile>        EnableFirewall
  Services\LanmanServer\Parameters                                 SMB1
  Control\FileSystem                                               NtfsDisableLastAccessUpdate
  Services\EventLog\<log>                                          MaxSize
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\PowerShell\1\ShellIds\Microsoft.PowerShell    ExecutionPolicy
  Software\Policies\Microsoft\Windows\PowerShell\…
  Software\Microsoft\Windows Script\Settings                       AmsiEnable
```

세 하이브를 모두 트랜잭션 로그와 함께 수집하십시오. 수집 직전에 변경된 설정은 로그에만 있을 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **저장된 구성은 실제로 적용되는 보호 상태가 아닙니다.** 수집된 레지스트리 값과 손상된 값 목록에 대한 진단
  행이 보고됩니다. 데이터가 없다는 것은 기본값, 수집되지 않은 출처, 또는 다른 관리 메커니즘을 반영할 수
  있으며, 보호 기능이 활성화되었거나 비활성화되었다는 증명이 아닙니다. `WeakensDefault`는 해당 설정이
  적용될 경우 명시된 기준을 낮출 수 있는 설정을 표시합니다.
- **레거시 Defender 플래그는 무시될 수 있습니다.** `DisableAntiSpyware`와 `DisableAntivirus`는 Defender가
  중지되었음을 증명하지 않습니다. 적용 여부는 플랫폼 버전, 온보딩 및 변조 방지(tamper protection)에 따라
  달라집니다. [Microsoft의 레거시 설정 설명](https://learn.microsoft.com/en-us/windows-hardware/customize/desktop/unattend/security-malware-windows-defender-disableantispyware)을 참조하십시오.
- **최신 관리형 제외 항목은 레지스트리 캡처에서 확인하지 못할 수 있습니다.** 플랫폼 4.18.25110.6부터
  Defender for Endpoint 구성 관리를 사용하는 장치에서는 제외 구성을 가져오려면 지원되는 Defender
  PowerShell cmdlet이 필요합니다. 따라서 오프라인 레지스트리에 없다고 해서 제외 항목이 없다고 확정할 수는
  없습니다.
  [Microsoft 구성 지침](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-settings).
- **약화되었다고 악의적인 것은 아닙니다.** 예:
  - `DisableAntiSpyware` = 1은 다른 바이러스 백신이 등록되어 있는 동안 Windows도 설정합니다.
  - 서버에서는 원격 데스크톱을 의도적으로 열어 둡니다.
  - Windows 7은 이벤트 로그 크기가 더 작았고 마지막 액세스 시간을 업데이트하지 않았습니다.
- **기본값은 버전에 따라 다릅니다.** 일부 값은 예전에는 기본값이었습니다.
  - `NtfsDisableLastAccessUpdate` = 1은 Windows 10 1803 이전의 기본값이었습니다. 이 값은 판정 없이
    보고되지만, 이후 버전에서 수동으로 설정한 `0x80000001`은 약화된 것으로 판정됩니다.
  - `RunAsPPL`, `LsaCfgFlags` 및 `UseLogonCredential`은 현재 Windows와 비교됩니다.
- **Defender 제외 항목:** 각 경로, 확장자, 프로세스 또는 주소가 하나의 레코드입니다. 모든 제외 항목은
  Defender를 약화시킵니다. Defender가 스스로 추가하는 `TemporaryPaths`에는 판정이 없습니다. 사용자가 쓰기
  가능한 폴더, `.ps1`/`.exe` 확장자 및 LOLBin(`rundll32.exe`, `powershell.exe`)을 찾으십시오.
- **정책 키**(`Policy` = True)는 그룹 정책, 도구, 그리고 정책이 로컬 설정보다 우선한다는 것을 아는 공격자가
  기록합니다. 도메인의 GPO와 상관 분석하십시오.
- **이벤트 로그 채널:** 보안 관련 채널 25개만 보고되며, Windows가 각 채널을 기본적으로 켜는지 여부도 함께
  표시됩니다. 예를 들어 `TaskScheduler/Operational`과 `DNS-Client/Operational`은 기본적으로 꺼져
  있습니다. 그 밖의 채널은 `parse-registry read`를 사용하십시오.
- **키 시간:** `KeyLastWrittenUTC`는 키의 시간이므로 반드시 이 값이 변경된 시점인 것은 아닙니다.
  `Policies` 키 전체가 한 번에 기록되는 경우가 흔합니다.
- **제어 집합:** 기본적으로 Windows가 마지막으로 사용한 제어 집합을 읽습니다. `--all-control-sets`는 모든
  제어 집합을 읽습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry security-settings <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM and NTUSER.DAT hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use
      --weakened-only     Only settings that weaken the default

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry security-settings -d evidence/ -o settings.csv
juntei parse-registry security-settings --weakened-only -d evidence/ -t jsonl \
  | jq -r '[.Host, .Area, .KeyPath, .ValueName, .Value, .Meaning] | @tsv'
```

요약에는 영역별 설정 수가 집계되며, 기본값을 약화시키는 설정과 모든 Defender 제외 항목이 나열됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 호스트, 하이브, 그리고 그 유형(`SOFTWARE`, `SYSTEM`, `NTUSER.DAT`)입니다. |
| `ControlSet`, `IsCurrentControlSet` | `SYSTEM`의 경우: 제어 집합과, 그것이 Windows가 마지막으로 사용한 제어 집합인지 여부입니다. `Select\Current`가 없거나 유효하지 않으면 알 수 없음입니다. |
| `User`, `UserSID` | `NTUSER.DAT`의 경우: 프로필 폴더 이름과, 시스템의 `ProfileList`에서 얻은 사용자의 SID입니다. |
| `Area` | `defender`, `uac`, `credentials`, `remote_access`, `network`, `powershell`, `evidence`입니다. |
| `Setting` | 설정의 종류입니다: `defender_disable_realtime_monitoring`, `defender_exclusion_path`, `uac_local_account_token_filter_policy`, `wdigest_use_logon_credential`, `rdp_deny_connections`, `powershell_script_block_logging`, `event_channel_enabled`, … |
| `Value` | 값입니다: 10진수 숫자, 문자열, 또는 제외 항목 자체입니다. 값이 설정의 유형과 다르면 비어 있습니다. |
| `Meaning` | 값의 의미입니다. |
| `WeakensDefault` | 해당 값이 없을 때 현재 Windows가 하는 동작을 약화시키면 `True`, 그렇지 않으면 `False`이며, 해당되지 않거나(크기, 포트, AMSI 공급자) 판단할 수 없으면 비어 있습니다. |
| `Policy` | 키가 `Policies` 키 아래에 있는지 여부입니다. |
| `KeyPath`, `ValueName` | 키와 값입니다. 제외 항목의 값 이름은 제외된 항목 자체이며, AMSI 공급자에는 값 이름이 없습니다. |
| `KeyLastWrittenUTC` | 키의 마지막 쓰기 시간입니다. |
| `DataRaw` | 값이 설정의 유형과 다른 경우 그 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 잘못된 유형의 값과 읽을 수 없는 키입니다. |

## 조사 전략 { #investigation-strategy }

1. 모든 호스트에서 `--weakened-only`로 실행하고 호스트끼리 비교하십시오. 한 호스트에만 있는 설정이
   두드러집니다.
2. 각 Defender 제외 항목을 호스트에서 발견된 도구 및 Defender 로그(이벤트 5007은 구성 변경을 기록함)와
   대조하십시오.
3. 키 시간을 로그온, 서비스 설치 및 프로세스 생성과 상관 분석하십시오. `reg.exe`, `Set-MpPreference`,
   `netsh advfirewall`, `powershell` 명령줄(이벤트 4688, Sysmon 1 및 13, PowerShell 4104)을
   찾으십시오.
4. 자격 증명 관련 설정(`WDigest`, `RunAsPPL`, `LocalAccountTokenFilterPolicy`,
   `DisableRestrictedAdmin`)의 경우 그 이후의 자격 증명 덤프와 pass-the-hash를 찾으십시오.
5. 기본값으로 되돌리기 위해 삭제된 값은 `parse-registry read --recover-deleted`를, 이전 제어 집합은
   `--all-control-sets`를 사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: Microsoft Defender 바이러스 백신의 제외 항목 구성 및 유효성 검사](https://learn.microsoft.com/en-us/defender-endpoint/configure-exclusions-microsoft-defender-antivirus)
- [Microsoft: 사용자 계정 컨트롤 설정 및 구성](https://learn.microsoft.com/en-us/windows/security/application-security/application-control/user-account-control/settings-and-configuration)
- [Microsoft: 추가 LSA 보호 구성](https://learn.microsoft.com/en-us/windows-server/security/credentials-protection-and-management/configuring-additional-lsa-protection)
- [Microsoft: about_Logging_Windows](https://learn.microsoft.com/en-us/powershell/module/microsoft.powershell.core/about/about_logging_windows)
- [MITRE ATT&CK T1562.001: Impair Defenses: Disable or Modify Tools](https://attack.mitre.org/techniques/T1562/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
