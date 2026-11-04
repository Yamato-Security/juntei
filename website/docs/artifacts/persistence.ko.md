# 레지스트리 자동 실행 { #registry-autostarts }

## 개요 { #what-they-are }

Windows는 많은 레지스트리 위치에서 프로그램을 시작하고 DLL을 로드합니다. 부팅 시, 로그온 시, 프로그램이
시작하거나 종료할 때, `cmd.exe`가 열릴 때, COM 클래스가 생성될 때, 또는 스풀러, LSA, netsh나 시간 서비스가
플러그인을 로드할 때가 그렇습니다. 공격자는 지속성, 권한 상승 및 DLL 인젝션을 위해 이러한 위치를 사용하며,
정상 소프트웨어도 이를 사용합니다. Juntei는 `SOFTWARE`, `SYSTEM`, `NTUSER.DAT` 및 `UsrClass.dat`에서 이
위치들을 읽어 하나의 테이블로 만듭니다. 프로그램, DLL 또는 패키지당 하나의 레코드를 기록하며, 각각에 MITRE
ATT&CK 기법을 태그로 붙입니다.

<span class="evidence-strength">가장 타당한 결론:</span> 하이브가 이 프로그램 또는 DLL을 시작하거나
로드하도록 구성했습니다. 이것이 프로그램이 실행되었음을 증명하지는 않습니다. 실행되려면 해당 메커니즘이
지정하는 조건(로그온, 디버깅 대상 프로그램의 시작, 클래스 생성)이 충족되어야 하며, 다른 실행 증거도
필요합니다.

## 위치 { #locations }

| 하이브 | 읽는 항목 |
|---|---|
| `SOFTWARE` | `Microsoft\Windows\CurrentVersion\Run`, `RunOnce`(+`Setup`), `RunOnceEx`, `RunServices(Once)` 및 `Policies\Explorer\Run`과 그 `Wow6432Node` 및 터미널 서버 설치 모드 사본, `Explorer\StartupApproved\Run`, `Run32` 및 `StartupFolder`, `Winlogon`(`Shell`, `Userinit`, `Taskman`, `AppSetup`, `GinaDLL`, … 및 `Notify`), `LoadAppInit_DLLs`와 함께 `Windows`의 `AppInit_DLLs`, `Image File Execution Options\*`의 `Debugger`(`UseFilter` 포함) 및 `VerifierDlls`, `SilentProcessExit\*`의 `MonitorProcess`, `Active Setup\Installed Components\*`의 `StubPath`, `NetSh`, `Command Processor`의 `AutoRun` |
| `SYSTEM` | `Control\Session Manager`(`BootExecute`, `SetupExecute`, `Execute`, `PendingFileRenameOperations`, `KnownDLLs`, `AppCertDlls`, `Environment`의 .NET 프로파일러 변수), `Control\Lsa`의 `Authentication`, `Security` 및 `Notification Packages`(+`OSConfig`), `SecurityProviders`, `Control\Print`의 모니터, 공급자 및 인쇄 프로세서, `Services\W32Time\TimeProviders` |
| `NTUSER.DAT` | 사용자의 `Run` 키와 `StartupApproved`, `Windows NT\CurrentVersion\Windows`의 `Load` 및 `Run`, `Winlogon`의 `Shell`, `Environment`(`UserInitMprLogonScript`, .NET 프로파일러), `Control Panel\Desktop`의 `SCRNSAVE.EXE`, `PackagedAppXDebug`, `Command Processor`의 `AutoRun`, `Software\Classes`(`UsrClass.dat`으로서) |
| `UsrClass.dat` | `CLSID\*\InprocServer32` 및 `LocalServer32`(및 `Wow6432Node`의 것), 프로그램 유형(`exefile`, `batfile`, `scrfile`, …)과 자동 권한 상승 프로그램이 시작하는 대상(`ms-settings`, `mscfile`, `Folder`, …)의 `shell\open\command`, `ActivatableClasses\…\DebugInformation` |

각 하이브를 트랜잭션 로그와 함께 수집하십시오. 사용자 하이브는 `Users\<name>\NTUSER.DAT`와
`Users\<name>\AppData\Local\Microsoft\Windows\UsrClass.dat`에 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **기본값도 보고됩니다.** 깨끗한 시스템의 레코드 대부분은 Windows 기본값입니다.
  - `Userinit`는 `userinit.exe`입니다.
  - `Shell`은 `explorer.exe`입니다.
  - `KnownDLLs`, LSA 패키지, 인쇄 모니터, netsh 도우미 및 Active Setup 구성 요소에는 모두 기본 항목이 있습니다.

  같은 버전의 깨끗한 빌드와 다른 점을 찾으십시오.
- **`StartupApproved`는 작업 관리자의 시작 프로그램 목록만 다룹니다.** `Run` 항목이나 시작 프로그램
  폴더의 파일이 그곳에서 비활성화되었는지, 그리고 언제 비활성화되었는지를 나타냅니다. 일치하는 `Run` 값이
  없는 항목도 "only in StartupApproved"라는 세부 정보와 함께 보고됩니다. 이는 제거된 자동 실행 항목을
  보여 줄 수 있습니다.
- **`OverridesMachineRegistration`에는 시스템의 `SOFTWARE` 하이브가 필요합니다.** 사용자의 COM 클래스나
  명령은 이 하이브와 대조되며, 이 하이브는 호스트의 `SYSTEM` 하이브와 같은 방식으로 찾습니다. 재정의는 COM
  하이재킹 패턴(T1546.015)입니다. OneDrive 및 기타 사용자별 설치도 아무것도 재정의하지 않는 자체 클래스를
  등록합니다.
- **`ImagePath`와 `Args`는 구분할 수 있는 경우에만 명령줄에서 분리됩니다.**
  - 따옴표로 묶인 프로그램은 닫는 따옴표에서 분리됩니다.
  - 폴더가 없는 첫 단어는 프로그램으로 간주됩니다(`cmd /c …`).
  - 그 외에는 프로그램 확장자로 끝나는 가장 짧은 시작 부분에서 분리됩니다.

  공백이 포함된 따옴표 없는 경로는 다른 프로그램(`C:\Program.exe`)이 존재하면 그것을 실행할 수 있습니다.
- **`GlobalFlag`가 `VerifierDlls`와 `MonitorProcess`의 `Enabled`를 결정합니다.** `VerifierDlls`는
  `0x100`이 있을 때만 로드됩니다. `MonitorProcess`는 `0x200`과 `SilentProcessExit\<program>`
  `ReportingMode`의 `0x1`이 있을 때만 실행됩니다. 모드 `2`(덤프) 또는 `4`(알림)는 아무것도 실행하지 않으며,
  `Details`에 모드의 플래그가 표시됩니다.
- **`KeyLastWrittenUTC`는 키의 마지막 쓰기 시간**이며, 값의 마지막 쓰기 시간이 아닙니다. 여러 값이 이를
  공유합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry persistence <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE, SYSTEM, NTUSER.DAT and UsrClass.dat hives
  -f, --file <FILE>      Parse one of them
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set of a SYSTEM hive, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry persistence -d collection/ -o persistence.csv
juntei parse-registry persistence -f NTUSER.DAT -t jsonl \
  | jq 'select(.Mechanism != "com_server" or .OverridesMachineRegistration == true)'
```

요약은 메커니즘별 레코드 수를 집계합니다. 또한 `Run` 값 없이 남아 있는 `StartupApproved\Run` 및 `Run32`
항목(시작 프로그램 폴더의 파일에는 잃을 값 자체가 없음)과, 머신의 등록을 재정의하는 사용자 COM 클래스 및
명령을 나열합니다.

모든 `parse-registry` 명령과 마찬가지로, `-C`를 사용하더라도 읽는 파일 중 하나를 `-o`로 지정하면
거부합니다. 여기에는 하이브, 그 트랜잭션 로그, 그리고 시스템의 SYSTEM 및 SOFTWARE 하이브가 경로, 심볼릭
링크 또는 하드 링크로 지정된 경우가 포함됩니다. 판단할 수 없는 경우에는 쓰지 않습니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 호스트 컨텍스트, 원본 하이브 및 그 유형입니다. |
| `User`, `UserSID` | 사용자 하이브의 경우 프로필 폴더와 SID입니다. SID는 `UsrClass.dat`의 루트 키 이름에서, 그렇지 않으면 시스템의 `ProfileList`에서 얻습니다. |
| `Mechanism` | `run`, `run_once`, `winlogon`, `ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `boot_execute`, `lsa_security_package`, `com_server`, `uac_bypass_command`, … |
| `AttackTechnique` | 해당하는 경우 MITRE ATT&CK 기법(`T1547.001`, `T1546.012`, …)입니다. |
| `KeyPath`, `ValueName` | 하이브 루트 아래의 키(기록된 그대로의 이름)와 값(비어 있으면 기본값)입니다. |
| `Data` | 값의 텍스트 또는 목록의 한 항목입니다. 읽을 수 없거나 `StartupApproved`만 해당 항목을 지정하는 경우 비어 있습니다. |
| `ImagePath`, `Args` | 구분할 수 있는 경우 프로그램 또는 DLL과 인수입니다(위 참조). |
| `Enabled`, `DisabledAtUTC` | `StartupApproved`, `LoadAppInit_DLLs`, `GlobalFlag`(`ReportingMode` 포함), `UseFilter`, `IsInstalled`, 시간 공급자의 `Enabled` 또는 `ScreenSaveActive`가 나타내는 상태입니다. 아무것도 나타내지 않으면 비어 있습니다. |
| `Details` | 추가 정보입니다. 보류 중인 이름 변경이 수행하는 작업, COM 클래스 이름, `ReportingMode`, `FilterFullPath`, Active Setup의 `ShellComponent`, … |
| `OverridesMachineRegistration` | 사용자의 COM 클래스 또는 명령의 경우, 머신이 같은 것을 등록하고 있는지 여부입니다. 시스템의 `SOFTWARE` 하이브가 없으면 비어 있습니다. |
| `WOW64` | `Wow6432Node` 아래에 있음, 즉 32비트 프로그램용입니다. |
| `KeyLastWrittenUTC` | 키의 마지막 쓰기 시간입니다. |
| `DataRaw` | 값이 텍스트가 아니거나 유효한 UTF-16이 아닌 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 잘못된 값 유형, 읽을 수 없는 데이터, 크기가 잘못된 `StartupApproved` 항목입니다. |

## 조사 전략 { #investigation-strategy }

1. 요약부터 시작하십시오.
   - 머신의 등록을 재정의하는 COM 클래스와 명령
   - `Run` 값이 없는 `StartupApproved` 항목
   - 항목이 거의 없는 메커니즘(`ifeo_debugger`, `silent_process_exit`, `appinit_dlls`, `appcert_dlls`, `uac_bypass_command`, `file_association`, `logon_script`, `dotnet_profiler`)
2. LSA 패키지, `BootExecute`, `KnownDLLs`, 인쇄 모니터 및 netsh 도우미를 같은 Windows 버전의 깨끗한
   빌드와 비교하십시오.
3. `ImagePath`를 파일 시스템($MFT, 해시 및 서명)과 대조하여 확인하고, 사용자 프로필, `ProgramData` 및
   `Temp` 경로를 점검하십시오.
4. `KeyLastWrittenUTC`를 Security 및 Sysmon 레지스트리 이벤트, 프로세스 생성, Prefetch 및 Amcache와 상관
   분석하십시오.
5. `parse-registry read --recover-deleted`를 사용하여 이 키들에서 제거된 자동 실행 항목을 찾으십시오.

## 추가 정보 { #further-details }

- [MITRE ATT&CK T1547 Boot or Logon Autostart Execution](https://attack.mitre.org/techniques/T1547/)
- [MITRE ATT&CK T1546 Event Triggered Execution](https://attack.mitre.org/techniques/T1546/)
- [Microsoft: Run 및 RunOnce 레지스트리 키](https://learn.microsoft.com/en-us/windows/win32/setupapi/run-and-runonce-registry-keys)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
