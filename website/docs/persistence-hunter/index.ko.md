# Persistence Hunter { #persistence-hunter }

지속성(persistence)이란 코드를 다시 실행되게 만드는 방법입니다. Windows가 시작될 때, 사용자가 로그인할 때,
애플리케이션이 열릴 때, 예약된 시간이 되었을 때 또는 다른 이벤트가 발생할 때 코드가 실행되도록 합니다.
공격자는 정상 소프트웨어가 업데이트, 장치 지원, 보안 및 관리에 사용하는 것과 동일한 설정을 악용할 수 있습니다.

`juntei persistence-hunter`는 수집된 Windows 증거를 오프라인으로 검사하며, 이러한 메커니즘과 관련
계정/보호 이상 징후를 다루는 **131개의 검사**를 수행합니다. 레지스트리 하이브, 작업 파일, 스크립트,
애플리케이션 설정 및 일부 데이터베이스 레코드를 읽습니다.

이 페이지는 구현된 모든 검사를 설명합니다. 전체 옵션, 출력 필드, 프로필 및 타임스탬프 처리에 대해서는
[명령어 가이드](../commands/persistence-hunter.md)를 참조하십시오.
[감사의 말](acknowledgements.md)에는 Python 프로젝트 PyrsistenceSniper와 원조인 PowerShell 도구
PersistenceSniper로의 링크가 있습니다.

## 조사 시작하기 { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

후보 전체 목록을 얻으려면 `--min-severity info`를 사용하십시오. 수동으로 실행하는 명령어의 기본값은
`medium`입니다. 해당 프로필은 페이로드나 서명자 증거가 없어 완전한 비교가 불가능한 경우에도 부분적인 허용
목록 일치를 `LOW`로 분류할 수 있습니다. 허용 목록과 일치한다고 해서, 또는 익숙하지 않은 항목이라고 해서
악성 여부가 결정되지는 않습니다. `auto-parse`는 이미 모든 심각도를 유지하며, 결과를 각 컴퓨터의
`persistence/` 범주 아래에 기록합니다.

특정 메커니즘에 집중하려면 아래에서 해당 **검사 ID**를 복사하십시오.

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

이 명령어는 기본적으로 공유 워커 풀을 사용합니다. 워커 4개를 사용하려면 `-T 4`를, 단일 스레드로
검사하려면 `-T 1`을 추가하십시오. 입력에는 여러 컴퓨터가 포함될 수 있으며, 각 컴퓨터의 증거는 분리되어 유지됩니다.

## 결과를 증거로 읽기 { #read-findings-as-evidence }

| 질문 | 확인할 내용 |
|---|---|
| 무엇이 어디에 구성되었는가? | `CheckId`, `Path`, 그리고 인수를 포함한 전체 `Value`를 확인하십시오. `SourceFile`을 따라 수집된 하이브, 파일 또는 데이터베이스를 찾으십시오. |
| 대상이 수집본에 존재하는가? | `ResolvedPath`, `ResolutionStatus` 및 `Exists`를 확인하십시오. 수집되지 않았거나, 모호하거나, 지원되지 않는 참조는 알 수 없음으로 처리됩니다. Juntei는 분석가 머신의 파일로 대체하지 않습니다. |
| 구성이 실제로 유효한가? | 실제 트리거, 활성화 플래그, 선택된 제어 집합, Windows/애플리케이션 버전, 사용자 및 아키텍처를 확인하십시오. 일부 검사는 의도적으로 비활성 구성도 유지합니다. |
| 이것이 공격인가? | `Severity`와 `PolicyReason`은 판정이 아니라 프로필 일치 결과를 설명합니다. 예상되는 소프트웨어, 전체 명령, 파일 해시, 독립적인 실행 증거 및 주변 타임라인과 비교하십시오. |
| 누가 실행할 수 있었는가? | `AccessGained`는 범위/컨텍스트에 대한 힌트입니다. 시스템 전체 구성은 흔히 `SYSTEM`으로 표시되지만, 이것이 실행된 프로세스가 SYSTEM으로 실행되었음을 증명하지는 않습니다. 가능한 경우 작업 주체(principal)와 BITS 소유자 SID가 더 구체적인 컨텍스트를 제공합니다. |
| 언제 변경되었는가? | `KeyLastWrite`는 레지스트리 키 전체의 시간입니다. `LastChange`, `ChangeSource` 및 `ChangeEvidence`는 가능한 경우 별도로 수행한 MFT/EVTX 상관 분석 결과를 나타냅니다. 어느 필드도 페이로드 실행 시간을 자동으로 나타내지는 않습니다. |
| 복구된 상태는 얼마나 신뢰할 수 있는가? | 재생, 카빙, 손상, 누락 또는 제한된 증거에 대해서는 `RecordStatus`, `Coverage.csv` 및 경고를 확인하십시오. 복구된 과거 레코드는 더 이상 활성 상태가 아닐 수 있습니다. |
| 명시된 게시자를 신뢰할 수 있는가? | `Signer`와 `SignerStatus`는 PE/인증서/카탈로그 메타데이터를 기술한 것입니다. Juntei는 Authenticode 서명이나 인증서 신뢰를 검증하지 않습니다. |

결과가 없다고 해서 지속성이 없다는 것이 증명되지는 않습니다. `complete` 커버리지 행은 해당하는 수집
원본을 읽었다는 의미일 뿐, 수집본에 관련된 모든 파일, 모든 사용자 또는 가능한 모든 Windows 지속성
메커니즘이 포함되어 있다는 의미가 아닙니다. 다른 커버리지 상태는 누락, 부분, 비활성화 또는 실패한 검사를
설명합니다. 이 헌터는 라이브 메모리, 네트워크 또는 클라우드 ID에 대한 검사 도구가 아닙니다.

## 카탈로그 및 위치 표기법 { #catalogue-and-location-notation }

이 카탈로그는 PyrsistenceSniper를 기반으로 하며, Juntei 고유의 `start_menu_com` 검사가 추가되어 있습니다.
아래 설명은 증거상의 제약을 포함하여 Juntei 고유의 동작을 기술합니다. **레지스트리 기반 검사 116개와
파일/저장소 검사 15개**가 있습니다. 일부 파일 검사는 레지스트리 데이터도 함께 결합하므로, 이 숫자는
상호 배타적인 수집 요구 사항이 아니라 구현상의 개수입니다. 아래에서는 메커니즘을 조사 목적별로 그룹화하고,
각 그룹 안에서는 검사 ID의 알파벳순으로 정렬했습니다.

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` 및 `HKLM\SAM`은 수집된 시스템 하이브를 가리키며, 일반적으로
  `Windows\System32\config`에서 가능한 경우 트랜잭션 로그와 함께 수집합니다.
- `HKU\<user>`는 해당하는 수집된 사용자 하이브를 의미합니다. 대부분의 사용자 설정은 NTUSER.DAT에서 가져오며,
  클래스 등록 정보는 명시된 경우 `AppData\Local\Microsoft\Windows\UsrClass.dat`에서 가져올 수 있습니다.
- `<ControlSet>`은 SYSTEM에서 선택된 활성 제어 집합을 의미합니다. DSRM 검사의 추가적인 과거 위치는
  아래에 명시되어 있습니다. 이를 검토할 때는 `SYSTEM\Select`를 확인하십시오.
- `<name>`은 가변적인 경로 구성 요소입니다. 레지스트리 대상 표에서 `<child>`는 바로 아래의 하위 등록 항목을
  의미하며, 무제한 재귀 탐색을 의미하지 않습니다. 명시적으로 재귀하는 검사는 탐색 방식을 별도로 설명합니다.
  Office 경로의 대괄호는 선택적인 버전을 나타냅니다.
- 표에는 검사가 선택하는 실제 시스템 및 사용자 뷰가 나열되어 있습니다. 32비트 뷰는 구현된 경우에만
  나열됩니다. 모든 검사가 가능한 모든 아키텍처/뷰를 검사한다고 가정하지 마십시오.
- 일반적인 값 검사는 비어 있지 않은 구성 항목을 출력하며, 해당하는 경우 다중 문자열 요소와 숫자 0도
  포함합니다. 이러한 검사 자체는 악성 페이로드, 활성화된 트리거 또는 성공적인 실행을 요구하지 않습니다.
  더 선택적인 조건은 검사별로 명시되어 있습니다.

ATT&CK ID는 카탈로그에서 물려받은 매핑이자 추가 학습을 위한 링크입니다. 이는 잠재적인 악용을 설명할 뿐,
공격자가 해당 기법을 수행했다는 판단이 아닙니다. 보호 설정 및 진단 검사는 독립적인 지속성 메커니즘이
아니라 이를 가능하게 하는 조건을 설명하는 것일 수 있습니다.

| 메커니즘 그룹 | 검사 |
|---|---:|
| [부팅, 로그인 및 시작](#boot-sign-in-and-startup) | 32 |
| [서비스 및 시스템 확장](#services-and-system-extensions) | 13 |
| [예약된 작업](#scheduled-tasks) | 3 |
| [WMI, BITS 및 기타 이벤트 트리거](#wmi-bits-and-other-event-triggers) | 11 |
| [탐색기, COM 및 파일 또는 URL 처리기](#explorer-com-and-file-or-url-handlers) | 15 |
| [Office 애플리케이션 및 추가 기능](#office-applications-and-add-ins) | 7 |
| [디버거, 크래시 처리기 및 호환성 shim](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [.NET 런타임 및 프로파일러 설정](#net-runtime-and-profiler-settings) | 7 |
| [DLL 및 실행 파일 조회 재정의](#dll-and-executable-lookup-overrides) | 18 |
| [계정 및 로그인 표시 여부](#accounts-and-sign-in-visibility) | 4 |
| [인증 구성 요소 및 자격 증명 보호](#authentication-components-and-credential-protection) | 8 |
| **합계** | **131** |

## 부팅, 로그인 및 시작 { #boot-sign-in-and-startup }

이 설정들은 부팅, 로그인 또는 데스크톱 초기화 중에 명령이나 라이브러리가 실행되도록 합니다. Windows 버전과 해당 사용자를 검토하십시오. 설정이 더 이상 사용되지 않거나, 비활성화되어 있거나, 공격자가 아니라 키오스크 용도로 구성된 것일 수 있습니다.

### Active Setup {#active_setup}

**검사 ID:** `active_setup` · **카탈로그 ATT&CK:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**Juntei가 검사하는 내용:** 네이티브 뷰와 32비트 뷰에서 시스템 전체 Active Setup 구성 요소 등록의 `StubPath`를 읽습니다. Windows는 사용자 프로필을 준비할 때 이 명령들을 사용할 수 있습니다. 이 검사는 리터럴 자리 표시자 값인 `U`와 `/UserInstall`은 건너뜁니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` 및 이에 대응하는 `Wow6432Node` 트리.

**조사 방법:** 각 구성 요소를 설치된 제품과 비교하고 명령 인수를 확인하십시오. 시스템 전체 등록은 로그인하는 사용자의 세션에서 실행으로 이어질 수 있습니다. 출력의 `SYSTEM` 레이블은 구성 범위를 나타낼 뿐, 실행된 프로세스 토큰을 증명하지 않습니다.

### Boot Execute {#boot_execute}

**검사 ID:** `boot_execute` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Session Manager의 `BootExecute` 다중 문자열을 읽고, 비어 있지 않은 개별 항목을 보존합니다. 초기 부팅 단계에 구성된 명령을 기록합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**조사 방법:** `autocheck autochk *`는 잘 알려진 기본값이며, 디스크 도구가 정상 항목을 추가할 수도 있습니다. 기본값이 아닌 항목이 모두 실행되었거나 악성이라고 가정하지 말고, 추가된 명령과 해당 파일을 조사하십시오.

### Boot Verification Program {#boot_verification_program}

**검사 ID:** `boot_verification_program` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 부팅 검증과 관련된 실행 파일 위치인 `BootVerificationProgram`의 `ImagePath`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**조사 방법:** 이 오래된 메커니즘이 수집된 Windows 버전과 부팅 구성에 적용되는지 확인하십시오. 대상 파일을 확인하고 그 용도를 예상되는 부팅 관리 소프트웨어와 비교하십시오.

### Explorer Load 값 {#explorer_load}

**검사 ID:** `explorer_load` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시스템 및 사용자 하이브에서 Windows 로그인 초기화 구성과 관련된 `Load` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**조사 방법:** 익숙하지 않은 명령, 사용자가 쓸 수 있는 대상 위치, 콘텐츠를 가져오거나 디코딩하는 인수를 조사하십시오. 적용 여부와 실행 여부는 별도로 확인하십시오. 결과가 비어 있다고 해서 다른 로그인 메커니즘까지 확인된 것은 아닙니다.

### 글꼴 드라이버(Font Drivers) {#font_drivers}

**검사 ID:** `font_drivers` · **카탈로그 ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei가 검사하는 내용:** 레거시 `Font Drivers` 레지스트리 키의 비어 있지 않은 값을 잠재적인 라이브러리 등록으로 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | 비어 있지 않은 모든 값 |

**조사 방법:** 이는 드물게 쓰이는 레거시 구성 영역으로 취급하십시오. 로드될 수 있다고 결론 내리기 전에 Windows 버전 지원 여부와 참조된 라이브러리를 확인하고, 실제 사용 여부는 프로세스 또는 이미지 로드 증거로 확증하십시오.

### 그룹 정책 확장 DLL {#gp_extension_dlls}

**검사 ID:** `gp_extension_dlls` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** Winlogon 아래 바로 하위에 있는 각 그룹 정책 확장 등록에서 `DllName`을 읽습니다. 이 DLL들은 정책 처리를 확장합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**조사 방법:** 관리 및 보안 제품이 확장을 설치할 수 있습니다. 실행을 주장하기 전에 확장 GUID와 DLL을 예상되는 제품과 대조하고, 정책 처리 및 이미지 로드 증거를 검토하십시오.

### 그룹 정책 스크립트 {#gp_scripts}

**검사 ID:** `gp_scripts` · **카탈로그 ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei가 검사하는 내용:** 수집된 로컬 그룹 정책의 `scripts.ini` 및 `psscripts.ini` 등록 파일(사용자별 로컬 정책 저장소 포함)에서 명령/매개 변수 쌍을 분석합니다. 시작/종료 및 로그온/로그오프 섹션의 구성된 명령과 인수를 유지하며, 시스템 섹션과 사용자 섹션을 구분합니다.

**검사하는 증거:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` 및 이에 해당하는 `GroupPolicy\User\Scripts` 파일.
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`.

**조사 방법:** 구현된 로컬 정책 등록 위치만 검사합니다. 이는 SYSVOL/GPO에 대한 완전한 감사가 아닙니다. 정책 디렉터리 아래의 다른 위치에 스크립트 파일이 있다는 것만으로는 충분하지 않습니다. 등록 정보를 정책 처리 및 실행 증거와 상관 분석하십시오.

### 로그온 스크립트(UserInitMprLogonScript) {#logon_scripts}

**검사 ID:** `logon_scripts` · **카탈로그 ATT&CK:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**Juntei가 검사하는 내용:** 해당 사용자의 로그인 시 실행할 스크립트를 지정할 수 있는 사용자의 `UserInitMprLogonScript` 환경 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**조사 방법:** 승인된 로그온 스크립트와 비교하고 실제 스크립트를 확인하십시오. 이 값은 이후의 로그인이나 실행을 증명하지 않으며, 도메인에서 제공하는 스크립트에는 추가적인 정책 증거가 필요합니다.

### Platform Execute {#platform_execute}

**검사 ID:** `platform_execute` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Session Manager의 비어 있지 않은 각 `PlatformExecute` 항목을 읽습니다. 플랫폼 관련 부팅 초기화에 지정된 프로그램을 식별합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**조사 방법:** 하드웨어 또는 프로비저닝 작업 흐름으로 항목이 설명될 수 있습니다. 값 이름만으로 공격을 추론하지 말고, 이 Windows 빌드의 부팅 경로가 이 값을 사용하는지 확인하고 참조된 이미지를 조사하십시오.

### RDP 클라이언트 확장 DLL {#rdp_clx_dll}

**검사 ID:** `rdp_clx_dll` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** RDP 관련 확장 위치일 수 있는 Terminal Server 기본 사용자 구성의 `ClxDllPath`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**조사 방법:** Windows/RDP 버전과 설치된 확장을 확인하십시오. 구성된 DLL은 이미지 로드 분석의 단서이며, 레지스트리 값만으로는 RDP 연결이나 DLL 로드가 발생했음을 보여 주지 않습니다.

### RDP 가상 채널 DLL {#rdp_virtual_channel}

**검사 ID:** `rdp_virtual_channel` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시스템 전체 Terminal Server Client의 `Default\AddIns` 등록 아래에 있는 값을 열거합니다. 바이너리 값은 가능한 경우 UTF-16LE로 디코딩하여 사용 가능한 확장 경로를 보존합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: 모든 값.

**조사 방법:** RDP 클라이언트와 장치 리디렉션 제품은 정상적으로 추가 기능을 등록합니다. 실제 DLL 또는 명령과 관련 클라이언트 세션을 확인하십시오. 추가 기능 아래의 모든 값이 실행 파일 경로인 것은 아닙니다.

### RDP WDS 시작 프로그램 {#rdp_wds_startup}

**검사 ID:** `rdp_wds_startup` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** RDP Windows Station 드라이버 구성에서 `StartupPrograms`를 읽습니다. 원격 세션 시작 시 실행되도록 구성된 프로그램을 식별합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**조사 방법:** `rdpclip`은 잘 알려진 항목입니다. 추가된 항목과 호스트의 RDP 구성을 조사하고, 세션/프로세스 기록을 사용하여 구성된 프로그램이 실제로 시작되었는지 판단하십시오.

### 레지스트리 Run 키 {#run_keys}

**검사 ID:** `run_keys` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시스템/사용자 하이브의 Run, RunOnce, RunEx, RunOnceEx 및 정책 Explorer Run 값과, 나열된 시스템 32비트 키를 열거합니다. RunEx/RunOnceEx 섹션은 범위 및 순환 검사를 적용하여 탐색합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | 비어 있지 않은 모든 값 |

**조사 방법:** 소프트웨어 업데이트 프로그램과 트레이 애플리케이션이 흔히 여기에 나타납니다. 실행 파일, 명령 인수 및 사용자 컨텍스트를 비교하십시오. RunOnce 항목은 사용 후 사라질 수 있으므로, 수집된 상태가 이전 로그인의 완전한 기록은 아닙니다.

### RunServices (레거시) {#run_services}

**검사 ID:** `run_services` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시스템 전체 레거시 `RunServices` 키의 값을 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | 비어 있지 않은 모든 값 |

**조사 방법:** 이는 과거의 구성 영역이며, 최신 Windows 빌드가 이를 실행한다는 보장은 없습니다. 대상과 운영 체제 적용 여부를 조사한 다음, 독립적인 실행 증거를 찾으십시오.

### RunServicesOnce (레거시) {#run_services_once}

**검사 ID:** `run_services_once` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시스템 전체 레거시 `RunServicesOnce` 키의 값을 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | 비어 있지 않은 모든 값 |

**조사 방법:** 비어 있지 않은 값은 구성상의 단서로 취급하십시오. 원본 Windows 버전이 이 메커니즘을 지원하는지, 그리고 배포 또는 호환성 소프트웨어로 해당 항목이 설명되는지 확인하십시오.

### S0 Initial Command {#s0_initial_command}

**검사 ID:** `s0_initial_command` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 세션 0 초기화 명령일 수 있는 Session Manager의 `S0InitialCommand`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**조사 방법:** 수집된 Windows 빌드와 부팅 상태에 적용되는지 확인하십시오. 구성된 명령은 부팅 시 실행에 대한 타임스탬프가 있는 기록이 아닙니다. 해당 파일을 확인하고 부팅/프로세스 텔레메트리와 상관 분석하십시오.

### Session Manager Execute {#session_manager_execute}

**검사 ID:** `session_manager_execute` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Session Manager에서 비어 있지 않은 `Execute` 항목을 읽어 세션 초기화에 지정된 프로그램을 드러냅니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**조사 방법:** 예상되는 이미지 기준선과 비교하고 비정상적인 네이티브 실행 파일이나 경로를 확인하십시오. 실행을 주장하기 전에 변경 이후 관련 부팅/세션이 발생했는지 확인하십시오.

### Session Manager SubSystems {#session_manager_subsystems}

**검사 ID:** `session_manager_subsystems` · **카탈로그 ATT&CK:** [T1547](https://attack.mitre.org/techniques/T1547/)

**Juntei가 검사하는 내용:** Windows 하위 시스템 명령을 구성하는 Session Manager `SubSystems` 키의 `Windows` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**조사 방법:** 정상 명령은 길고 버전에 따라 다릅니다. 전체 값과 인수를 같은 빌드의 신뢰할 수 있는 기준선과 비교하십시오. 중요한 하위 시스템 설정을 변경하면 시작 자체가 실패할 수도 있습니다.

### Setup Execute {#setup_execute}

**검사 ID:** `setup_execute` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 시작 시 실행되도록 구성된 설치 관련 명령을 다루는 Session Manager의 `SetupExecute` 항목을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**조사 방법:** 설치, 업그레이드 및 복구 작업이 정상 항목을 만들 수 있습니다. 구성을 지속성으로 판단하기 전에 설치 로그 및 참조된 이미지와 비교하십시오.

### Shell Folders 시작 프로그램 리디렉션 {#shell_folders_startup}

**검사 ID:** `shell_folders_startup` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Shell Folders와 User Shell Folders 모두의 `Startup` 및 `Common Startup`을 기본 시작 프로그램 위치와 비교합니다. 기본값이 아닌 리디렉션을 보고하고, 수집된 해당 폴더의 내용을 검사합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` 및 `User Shell Folders`: `Common Startup`.
- NTUSER.DAT의 이에 해당하는 `Software\Microsoft\Windows\CurrentVersion\Explorer` 설정: `Startup`, 그리고 리디렉션된 위치에서 수집된 파일.

**조사 방법:** 리디렉션된 시작 프로그램 폴더는 자동 시작을 숨길 수 있지만, 리디렉션 자체는 정상일 수 있습니다. 경로는 수집된 증거 안에서 확인되어야 합니다. 수집되지 않은 대상은 알 수 없음으로 처리되며, 폴더가 비어 있었다는 증거가 아닙니다.

### Shell Launcher 재정의 {#shell_launcher}

**검사 ID:** `shell_launcher` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** SOFTWARE에서 정책 `Shell` 값과 `system.ini` boot Shell 매핑을 읽습니다. 이들은 셸을 선택하는 대체 경로입니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**조사 방법:** 키오스크와 애플리케이션 어플라이언스는 의도적으로 Explorer를 대체할 수 있습니다. 선택된 셸을 머신의 역할과 비교하고, 해당 설정이 관련 세션에서 유효했는지 확인하십시오.

### 시작 프로그램 폴더 {#startup_folder}

**검사 ID:** `startup_folder` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 기본 사용자별 및 공용 시작 프로그램 폴더 바로 아래의 파일을 검사하며, 처음으로 확인되는 시작 프로그램 폴더 리디렉션을 따릅니다. 수집된 `.lnk`를 분석할 수 있으면 바로 가기의 대상을 복원합니다.

**검사하는 증거:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` 및 `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`.
- 시스템/사용자 Shell Folders 및 User Shell Folders 설정에서 해당하는 `Common Startup` / `Startup` 리디렉션.

**조사 방법:** 시작 프로그램의 링크와 파일은 로그인 시 정상 애플리케이션을 실행할 수 있습니다. 파일 이름뿐 아니라 대상 인수도 확인하십시오. 공용 폴더는 공유 구성 위치일 뿐이며, 실행된 프로그램이 SYSTEM 토큰을 사용했음을 증명하지 않습니다.

### 터미널 서비스 초기 프로그램 {#ts_initial_program}

**검사 ID:** `ts_initial_program` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 터미널 서비스 정책의 `InitialProgram`과 시스템 Terminal Server 설치용 Run/RunOnce/RunOnceEx 구성(범위가 제한된 중첩 RunOnceEx 섹션 포함)을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | 비어 있지 않은 모든 값 |

**조사 방법:** 원격 애플리케이션 배포에서 이러한 설정을 의도적으로 사용할 수 있습니다. 영향을 받는 세션을 확인하고 지정된 명령을 프로세스 증거까지 추적하십시오. 등록 정보만으로는 원격 로그온이 있었음을 알 수 없습니다.

### Winlogon AppSetup {#winlogon_appsetup}

**검사 ID:** `winlogon_appsetup` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** Winlogon에서 `AppSetup`을 읽고, 쉼표로 구분된 항목을 개별 후보로 나눕니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**조사 방법:** 각 구성 요소를 예상되는 Windows 설치/로그인 소프트웨어와 비교하십시오. 목록의 각 항목이 별도로 관찰된 실행은 아니며, 관련성은 Windows 버전과 배포 상태에 따라 달라질 수 있습니다.

### Winlogon GinaDLL {#winlogon_gina_dll}

**검사 ID:** `winlogon_gina_dll` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 레거시 Winlogon `GinaDLL` 대체 설정을 읽습니다. GINA는 이전 Windows의 로그인 확장 모델이었습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**조사 방법:** 이는 주로 Vista 이전 시스템과 관련이 있습니다. 최신 시스템에 남아 있는 키는 유효한 로그온 확장의 증거가 아닙니다. 원본 버전과 참조된 DLL을 확인하십시오.

**추가 정보:** [Microsoft 문서](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina).

### Winlogon MPNotify {#winlogon_mpnotify}

**검사 ID:** `winlogon_mpnotify` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 로그인 관련 확장 영역인 Winlogon의 `mpnotify` 명령/라이브러리 설정을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**조사 방법:** 대상과 인수를 일치하는 Windows 기준선과 비교하십시오. 코드가 실행되었다고 주장하기 전에 버전별 동작을 확인하고 이후의 로그인과 상관 분석하십시오.

### Winlogon Notify 패키지 {#winlogon_notify_packages}

**검사 ID:** `winlogon_notify_packages` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** Winlogon `Notify` 바로 아래의 각 패키지 등록에서 `DllName`을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**조사 방법:** 알림 패키지는 레거시 확장 메커니즘입니다. 수집된 Windows 버전이 이를 로드하는지 확인하고, 승인된 인증 소프트웨어와 설명되지 않는 라이브러리를 구분하십시오.

### Winlogon Shell {#winlogon_shell}

**검사 ID:** `winlogon_shell` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 시스템 전체 및 사용자별 Winlogon `Shell` 값을 읽어 데스크톱 셸 선택을 드러냅니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**조사 방법:** 일반적으로는 Explorer이지만, 키오스크는 다른 셸을 사용할 수 있습니다. 추가된 명령이나 비정상적인 경로를 조사하고, 로그인 시 어떤 사용자의 유효 구성이 적용되었는지 확인하십시오.

### Winlogon System {#winlogon_system}

**검사 ID:** `winlogon_system` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** Winlogon의 `System` 값을 읽고, 쉼표로 구분된 항목을 개별 후보로 나눕니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**조사 방법:** 이를 버전에 민감한 로그인 구성 단서로 취급하십시오. 목록을 실행 기록으로 간주하지 말고, 참조된 모든 구성 요소를 확인하고 실행 증거로 확증하십시오.

### Winlogon Taskman {#winlogon_taskman}

**검사 ID:** `winlogon_taskman` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 대체 작업 관리 명령을 지정하는 시스템/사용자 Winlogon `Taskman` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**조사 방법:** 어떤 Windows 동작이 이 설정을 사용하는지, 그리고 대상이 예상된 것인지 확인하십시오. 대체 항목이 구성되어 있다고 해서 사용자가 해당 기능을 호출했다는 증거는 아닙니다.

### Winlogon Userinit {#winlogon_userinit}

**검사 ID:** `winlogon_userinit` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 시스템/사용자 Winlogon `Userinit` 값을 읽고, 쉼표로 구분된 프로그램을 나누되 끝의 빈 항목은 무시합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**조사 방법:** 정상 항목은 대개 `userinit.exe`를 가리킵니다. 추가된 항목과 변경된 경로를 확인하십시오. 해당 사용자와 이후의 로그인을 확인하십시오. `KeyLastWrite`는 이 값이 아니라 키 전체에 대한 것입니다.

### Winlogon VMApplet {#winlogon_vmapplet}

**검사 ID:** `winlogon_vmapplet` · **카탈로그 ATT&CK:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**Juntei가 검사하는 내용:** 이전 시스템 애플릿 실행 영역에 해당하는 Winlogon의 `VMApplet` 명령 설정을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**조사 방법:** 운영 체제 적용 여부와 예상되는 애플릿을 확인하십시오. 예상하지 못한 명령은 조사할 가치가 있지만, 이를 트리거하는 인터페이스가 열렸다는 사실을 입증하지는 않습니다.

## 서비스 및 시스템 확장 { #services-and-system-extensions }

Windows 서비스와 확장 가능한 시스템 구성 요소는 실행 파일이나 DLL을 로드하는 정상적인 방법을 다양하게 제공합니다. 구성된 대상을 예상되는 제품, 서비스 계정, 시작 설정 및 원본으로 뒷받침되는 설치 이력과 비교하십시오.

### DiagTrack DLL {#diagtrack_dll}

**검사 ID:** `diagtrack_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** DiagTrack 서비스의 `ImagePath`를 읽습니다. 카탈로그상 이름은 DLL 중심이지만, 이 검사는 구성된 서비스 이미지 명령을 검사합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**조사 방법:** 서비스 기준선 및 일반 서비스 검사와 비교하십시오. 같은 구성이 둘 이상의 검사 결과를 만들 수 있으므로, 후보 개수는 서로 다른 공격의 개수가 아닙니다.

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**검사 ID:** `diagtrack_listener_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** DiagTrack-Listener autologger 구성에서 `FileName`을 읽습니다. 이는 ETW autologger 설정이며, DLL이 로드되었다는 증거가 아닙니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**조사 방법:** 원시 값과 Windows 로깅 구성을 확인하십시오. 카탈로그는 이를 DLL 기법으로 분류하지만, 이 조회는 로드 가능한 DLL인지 검증하지 않으며 이 값을 통한 실행을 증명하지도 않습니다.

### Netsh 도우미 DLL {#netsh_helper}

**검사 ID:** `netsh_helper` · **카탈로그 ATT&CK:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**Juntei가 검사하는 내용:** 시스템 `NetSh` 도우미 등록 키 아래의 비어 있지 않은 값을 열거합니다. 도우미는 DLL 기반 명령으로 `netsh.exe`를 확장할 수 있습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | 비어 있지 않은 모든 값 |

**조사 방법:** 네트워크 제품이 도우미를 설치할 수 있습니다. 각 값을 예상되는 제품과 대조하고 확인된 라이브러리를 검사한 다음, 관련 netsh 실행이나 이미지 로드를 찾으십시오.

### 인쇄 모니터(Print Monitors) {#print_monitors}

**검사 ID:** `print_monitors` · **카탈로그 ATT&CK:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**Juntei가 검사하는 내용:** 활성 제어 집합에서 바로 하위에 있는 각 Print Monitor 등록 아래의 `Driver`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**조사 방법:** 인쇄 소프트웨어와 포트 모니터는 정상적으로 스풀러를 확장합니다. DLL과 제품 메타데이터를 프린터 목록과 비교하고, 스풀러/이미지 로드 증거로 로드 여부를 확인하십시오.

### 인쇄 프로세서(Print Processors) {#print_processors}

**검사 ID:** `print_processors` · **카탈로그 ATT&CK:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**Juntei가 검사하는 내용:** x64 및 x86 인쇄 프로세서 등록 트리 아래의 `Driver`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**조사 방법:** 서드파티 프린터 제품군은 잘 알려진 Windows 기본값 외에 프로세서를 추가할 수 있습니다. 악의적인 의도로 판단하기 전에 두 아키텍처, 실제 DLL 및 설치된 프린터 소프트웨어를 검토하십시오.

### 서비스 제어 관리자 확장 {#scm_extension}

**검사 ID:** `scm_extension` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 서비스 관리자 확장 영역인 `ServiceControlManagerExtension`의 비어 있지 않은 값을 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | 비어 있지 않은 모든 값 |

**조사 방법:** 지정된 라이브러리와 이 Windows 빌드가 해당 설정을 사용하는지 확인하십시오. 부팅/서비스 활동과 상관 분석하십시오. 이 검사는 등록 정보의 목록을 작성할 뿐 확장이 로드되었음을 증명하지 않습니다.

### DNS 서버 수준 플러그인 DLL {#server_level_plugin_dll}

**검사 ID:** `server_level_plugin_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 활성 SYSTEM 제어 집합에서 DNS 서비스의 `ServerLevelPluginDll` 설정을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**조사 방법:** 이는 일부 도메인 컨트롤러를 포함하여 DNS 서버 역할을 실행하는 시스템과 관련이 있습니다. DLL, 서비스 계정 및 서비스 재시작/로드 증거를 조사하십시오. 해당 역할이 비활성 상태이거나 없으면 결론이 달라집니다.

### 서비스 실패 명령 {#service_failure_command}

**검사 ID:** `service_failure_command` · **카탈로그 ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei가 검사하는 내용:** 활성 제어 집합에서 각 서비스의 `FailureCommand`를 읽어 구성된 복구 명령을 드러냅니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`.

**조사 방법:** 이 명령은 서비스의 복구 동작이 이를 선택하고 해당 실패가 발생했을 때만 실행됩니다. 복구 구성과 실패/프로세스 기록을 확인하십시오. 값만으로는 활성화된 복구 동작이 있음을 입증하지 못합니다.

### SNMP 확장 에이전트 DLL {#snmp_extension_agent}

**검사 ID:** `snmp_extension_agent` · **카탈로그 ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei가 검사하는 내용:** SNMP 서비스의 `ExtensionAgents` 등록 키 아래의 값을 열거합니다. 값은 DLL의 직접 경로가 아니라 다른 레지스트리 위치를 가리킬 수 있습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | 비어 있지 않은 모든 값 |

**조사 방법:** 간접 등록은 직접 라이브러리까지 추적하고, SNMP가 설치되어 사용되고 있는지 확인하십시오. 이 출력에서 확인되지 않은 문자열은 DLL이 없거나 악성이라는 증거가 아닙니다.

### 시간 공급자(Time Providers) {#time_providers}

**검사 ID:** `time_providers` · **카탈로그 ATT&CK:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**Juntei가 검사하는 내용:** Windows Time `TimeProviders` 바로 아래의 등록에서 `DllName`을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**조사 방법:** 공급자 활성화 설정과 Windows Time 서비스 상태는 별도로 검토하십시오. DLL 경로 검사는 공급자가 활성화되어 있을 것을 요구하지 않습니다. 시간 동기화 제품이 정상적인 대체 항목을 제공할 수 있습니다.

### Windows 서비스(ServiceDll) {#windows_service_dll}

**검사 ID:** `windows_service_dll` · **카탈로그 ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei가 검사하는 내용:** 활성 SYSTEM 제어 집합의 서비스 아래에서 `Parameters\ServiceDll`을 읽고, 가능한 경우 서비스별 타임스탬프 증거를 첨부합니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`.

**조사 방법:** `svchost.exe` 같은 공유 호스트는 페이로드 DLL이 아닙니다. 라이브러리, 서비스 계정, 시작 상태 및 일치하는 서비스 설치/이미지 로드 증거를 확인하십시오. 등록 정보만으로는 서비스가 실행되었음을 증명하지 못합니다.

**추가 정보:** [Microsoft 문서](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows 서비스(ImagePath) {#windows_service_image_path}

**검사 ID:** `windows_service_image_path` · **카탈로그 ATT&CK:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**Juntei가 검사하는 내용:** 활성 제어 집합에서 서비스의 `ImagePath` 명령을 읽습니다. ImagePath가 없는 드라이버 유형 1, 2 또는 8은 암묵적인 `System32\drivers\<service>.sys` 대상을 유지합니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` 및 관련 서비스 메타데이터.

**조사 방법:** 서비스 계정과 시작 설정은 출력의 시스템 범위 레이블과 별도로 검토하십시오. 비활성화되었거나 요청 시 시작되는 등록도 여전히 목록에 포함됩니다. 타임스탬프에는 모호한 공유 실행 파일 경로보다 서비스 이름 기반 증거를 우선합니다.

**추가 정보:** [Microsoft 문서](https://docs.microsoft.com/en-us/windows/win32/services/service-programs).

### Windows Update 서비스 시작 DLL {#wu_service_startup_dll}

**검사 ID:** `wu_service_startup_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 활성 제어 집합에서 Windows Update 서비스의 `Parameters\ServiceDll`을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**조사 방법:** 라이브러리를 같은 Windows 빌드 및 설치된 서비스 상태와 비교하십시오. 이는 일반 ServiceDll 목록과 중복되며, 별개의 두 번째 침해를 의미하지 않습니다.

## 예약된 작업 { #scheduled-tasks }

작업 스케줄러는 구성을 XML 파일과 SOFTWARE 하이브 양쪽에 저장합니다. 두 원본을 함께 읽으십시오. 작업은 실행되지 않은 채 등록되어 있을 수 있으며, 불완전한 수집본은 숨겨진 작업이나 고스트 작업처럼 보일 수 있습니다.

### 고스트 예약 작업 {#ghost_task}

**검사 ID:** `ghost_task` · **카탈로그 ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei가 검사하는 내용:** SOFTWARE TaskCache의 `Tree`와 `Tasks` 등록을 비교하고, 수집본에 작업 파일이 있으면 그에 대응하는 작업 XML 파일을 찾습니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` 및 `TaskCache\Tasks`.
- 파일 시스템 비교를 위해 수집된 `Windows\System32\Tasks\<task path>` 파일.

**조사 방법:** 대응 항목이 없는 것은 삭제되었거나 오래된 작업, 기본 제공 등록 또는 불완전한 수집 때문일 수 있습니다. 수집된 XML이 없으면 파일 시스템 비교를 할 수 없습니다. 작업이 숨겨졌거나 악성이라고 판단하기 전에 커버리지와 정확한 불일치 내용을 확인하십시오.

### 숨겨진 예약 작업 {#hidden_scheduled_task}

**검사 ID:** `hidden_scheduled_task` · **카탈로그 ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei가 검사하는 내용:** TaskCache Tree 등록에서 `SD` 값이 없거나 바이너리 보안 설명자가 비어 있는지 검사합니다. 이는 일반적인 열거에서 작업을 숨기는 것과 관련된 조건입니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: `SD`의 존재 여부/데이터 및 대응하는 TaskCache 메타데이터.

**조사 방법:** SD가 없거나 비어 있는 것은 특정 구성 이상 징후일 뿐, 작업이 실행되었거나 악성이라는 증거가 아닙니다. XML의 `Hidden` 설정과 임의의 ACL 문제는 다른 조건이며, 여기서 사용하는 판단 조건이 아닙니다.

**추가 정보:** [Microsoft 문서](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/).

### 예약된 작업(XML 파일) {#scheduled_task_files}

**검사 ID:** `scheduled_task_files` · **카탈로그 ATT&CK:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**Juntei가 검사하는 내용:** 수집된 작업 XML에서 Exec 명령/인수와 COM 처리기 클래스 ID/데이터를 분석합니다. `Actions Context`/`Principal id`로 주체를 선택하고, 사용자와 그룹 신원을 분리하여 유지하며, SYSTEM, USER 또는 UNKNOWN으로 보고합니다. 확인되지 않은 COM 동작도 계속 표시됩니다.

**검사하는 증거:**

- Exec, ComHandler 및 사용 가능한 주체 필드를 포함한 `Windows\System32\Tasks\<task path>` XML 파일.
- 일치하는 사용자의 수집된 `UsrClass.dat`와 NTUSER `Software\Classes`, 그리고 시스템 SOFTWARE의 `Classes\CLSID`(네이티브 및 32비트 뷰).
- SID 주체를 위해 수집된 SOFTWARE `ProfileList`. 이름으로 지정된 주체는 고유하게 일치하는 `user.DOMAIN` 폴더를 우선하고, 다음으로 수집된 로컬 호스트 접미사, 마지막으로 일반 `user` 폴더를 사용합니다. 일치 결과는 휴리스틱임을 명시적으로 표시하며, 실제로 모호한 경우는 확인되지 않은 상태로 남겨 결과와 커버리지에 기록합니다. 휘발성 레지스트리 키는 필요하지 않습니다.
- 관리 코드의 `CodeBase`(버전 하위 키 포함), AppID 연결, 그리고 AppID `LocalService`에 대한 SYSTEM 서비스 `ImagePath`.

COM 서버 후보는 `Description`에 레지스트리 원본을 포함합니다. 실행 파일 이름만 있는
AppID 연결은 추측한 디렉터리를 할당하지 않고 확인되지 않은 상태로 둡니다. 로컬
`file:` URL은 디코딩하며, 원격 URL은 네트워크에 접근하지 않고 그대로 유지합니다. 사용자 클래스 재정의는
해당 프로필로 범위가 제한되며, 불완전한 사용자 CLSID가 그에 의해 가려진 시스템 등록으로 넘어가지 않습니다.
그룹만 지정된 작업은 개별 사용자에 대한 제약을 설명에 표시한 채 시스템 후보를 유지하며,
커버리지 오류 요약에 일상적인 알림을 추가하지 않습니다.

**조사 방법:** 원본 XML과 이벤트 로그에서 트리거, 활성화 상태, 주체 및 실행 기록을 확인하십시오. 생성된 작업이 한 번도 실행되지 않았을 수 있습니다. 확인되지 않은 COM 처리기도 여전히 후보이며, 주체 데이터가 없거나 그룹만 지정되어 있다고 해서 SYSTEM이라고 가정해서는 안 됩니다.

## WMI, BITS 및 기타 이벤트 트리거 { #wmi-bits-and-other-event-triggers }

WMI(Windows Management Instrumentation)는 시스템 관리를 지원합니다. BITS(Background Intelligent Transfer Service)는 백그라운드 전송을 지원합니다. 두 가지 모두 이벤트로 트리거되는 명령을 보관할 수 있습니다. 일부 메커니즘은 관리 이벤트, 전송 알림, 애플리케이션 실행 또는 사용자 동작을 기다립니다. 저장된 트리거는 유용한 구성 증거이지만, 그 존재만으로 트리거가 발동했다는 사실이 입증되지는 않습니다.

### 접근성 기능 백도어 {#accessibility_tools}

**검사 ID:** `accessibility_tools` · **카탈로그 ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei가 검사하는 내용:** 수집된 참조 바이너리에서 사용 가능한 Microsoft 카탈로그 메타데이터를 확보한 후, 수집된 System32 접근성 바이너리 7개를 PE OriginalFilename과 인증서/카탈로그 메타데이터로 비교합니다.

**검사하는 증거:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` 및 `DisplaySwitch.exe`.
- 수집된 `Windows\System32\winlogon.exe`, `cmd.exe` 또는 `notepad.exe`의 참조 메타데이터 및 해당하는 수집된 카탈로그.

**조사 방법:** 메타데이터는 이름을 바꿔 대체한 파일을 시사할 수 있지만, Juntei는 서명이나 인증서 신뢰를 검증하지 않습니다. 필요한 카탈로그가 없는 부족한 수집본에서는 서명되지 않은 것처럼 보이는 모든 파일을 백도어로 취급하는 대신 비교 커버리지가 불완전한 것으로 처리합니다.

### 접근성 기능 {#assistive_technology}

**검사 ID:** `assistive_technology` · **카탈로그 ATT&CK:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**Juntei가 검사하는 내용:** 네이티브/32비트 SOFTWARE 뷰에서 등록된 접근성 `StartExe` 및 `StartParams` 명령을 읽고, 사용자 및 DEFAULT 하이브에서 활성화된 접근성 `Configuration` 항목을 읽습니다. 숫자/문자열이 아닌 StartExe 자리 표시자는 건너뜁니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` 및 `StartParams`. 시스템 `Wow6432Node` 뷰도 포함합니다.
- NTUSER.DAT의 `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` 및 DEFAULT 하이브의 동등한 설정.

**조사 방법:** 접근성 소프트웨어는 정상적인 것입니다. 활성화된 기능 이름과 실제 실행 파일 등록을 구분한 다음, 대상과 로그인/데스크톱 실행 증거를 확인하십시오. 이는 접근성 실행 파일 자체를 교체하는 것과는 별개입니다.

**추가 정보:** [기법 참고 자료(hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/).

### BITS 작업 알림 명령 {#bits_notify_command}

**검사 ID:** `bits_notify_command` · **카탈로그 ATT&CK:** [T1197](https://attack.mitre.org/techniques/T1197/)

**Juntei가 검사하는 내용:** 수집된 `qmgr.db`, `qmgr0.dat` 또는 `qmgr1.dat`에서 길이가 표시된 문자열(counted string) 형식의 BITS 작업 레코드를 복구하며, 비어 있지 않은 알림 프로그램, 매개 변수, 작업 ID 및 소유자 SID를 유지합니다.

**검사하는 증거:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` 및 `qmgr1.dat`.

**조사 방법:** 이는 카빙된 레코드입니다. 할당된/현재 작업 상태는 검증되지 않으며 오래되었거나 삭제된 작업이 나타날 수 있습니다. 데이터베이스의 MFT 쓰기 시간은 컨테이너에 대한 약한 증거일 뿐이며, 알림이 실행된 시간이 아닙니다. 소유자 SID로 SYSTEM, 로컬 서비스, 네트워크 서비스 및 사용자를 구분할 수 있습니다.

### Command Processor AutoRun {#cmd_autorun}

**검사 ID:** `cmd_autorun` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 명령 프롬프트 시작에 영향을 줄 수 있는 설정인 시스템/사용자 Command Processor `AutoRun` 명령을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**조사 방법:** 개발 도구와 셸 사용자 지정으로 값이 설명될 수 있습니다. `cmd.exe /d`는 AutoRun을 우회하므로, 구성만으로는 cmd.exe가 나타날 때마다 AutoRun이 실행되었다고 증명할 수 없습니다. 명령 내용과 트리거한 프로세스의 인수를 확인하십시오.

**추가 정보:** [Microsoft 문서](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd).

### ErrorHandler.cmd 지속성 {#error_handler_cmd}

**검사 ID:** `error_handler_cmd` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 수집된 `Windows\System32\ErrorHandler.cmd` 스크립트의 존재를 오류 처리기 지속성 후보로 보고합니다.

**검사하는 증거:**

- `Windows\System32\ErrorHandler.cmd`.

**조사 방법:** 존재만으로는 해당 Windows 복구/오류 경로가 이를 호출했다는 것이 증명되지 않습니다. 스크립트를 읽고 출처를 확인한 다음, 설치/오류/프로세스 증거와 상관 분석하십시오.

### Power Automate Desktop 흐름 {#power_automate}

**검사 ID:** `power_automate` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 각 사용자의 Power Automate Desktop `flows` 및 `scripts` 저장소 아래에서 수집된 흐름/스크립트 디렉터리를 찾고, 발견된 각 흐름을 한 번씩 보고합니다.

**검사하는 증거:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` 및 `scripts\<flow>\...`.

**조사 방법:** 이 검사는 저장된 자동화를 식별할 뿐, 모든 예약 트리거나 실행된 흐름을 식별하지는 않습니다. 자동 실행 또는 악성 지속성으로 취급하기 전에 흐름 내용과 외부 실행 구성을 확인하십시오.

### PowerShell 프로필 {#powershell_profiles}

**검사 ID:** `powershell_profiles` · **카탈로그 ATT&CK:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**Juntei가 검사하는 내용:** 사용자 Documents, 발견된 OneDrive Documents, 수집된 Documents 리디렉션을 포함하여, 알려진 Windows PowerShell 및 PowerShell 설치/프로필 디렉터리에서 profile.ps1 형태의 파일을 찾습니다.

**검사하는 증거:**

- `Windows\System32\WindowsPowerShell\v1.0` 및 `Windows\SysWOW64\WindowsPowerShell\v1.0` 바로 아래에 있는, 이름이 `profile.ps1`로 끝나는 파일.
- 수집된 Documents 리디렉션과 발견된 OneDrive Documents 폴더를 포함한 사용자별 `Documents\WindowsPowerShell` 및 `Documents\PowerShell`.
- `Program Files\PowerShell` 또는 `Program Files (x86)\PowerShell` 아래의 버전 디렉터리 바로 아래에 있는, 이름이 `profile.ps1`로 끝나는 파일.

**조사 방법:** 프로필에는 무해한 사용자 지정이 들어 있을 수 있습니다. 스크립트를 읽고 호출되는 페이로드를 추적하십시오. `-NoProfile`로 PowerShell을 실행하면 프로필을 건너뜁니다. 파일의 존재만으로는 프로필 실행이나 악성 콘텐츠를 증명할 수 없습니다.

### 화면 보호기 하이재킹 {#screensaver}

**검사 ID:** `screensaver` · **카탈로그 ATT&CK:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**Juntei가 검사하는 내용:** 각 사용자의 Desktop `SCRNSAVE.EXE` 값을 읽어 구성된 화면 보호기 이미지를 식별합니다.

**검사하는 증거:**

- NTUSER.DAT의 `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE`.

**조사 방법:** 승인된 화면 보호기와 비교하고 대상을 확인하십시오. 트리거 가능 여부는 다른 데스크톱 설정, 세션 상태 및 시간 제한에 따라 결정됩니다. 저장된 경로는 화면 보호기가 시작되었음을 증명하지 않습니다.

### 원격 분석 컨트롤러 명령 {#telemetry_controller}

**검사 ID:** `telemetry_controller` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** TelemetryController와 그 바로 아래 하위 등록에서 `Binary`, `Command` 및 `MaintenanceCommand`를 읽습니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` 및 바로 아래 하위 항목: `Binary`, `Command`, `MaintenanceCommand`.

**조사 방법:** 선택된 실행 파일과 인수를 같은 Windows 빌드 및 유지 관리 소프트웨어와 비교하십시오. 레지스트리 명령은 유지 관리 트리거가 발동했음을 보여 주지 않습니다.

### Windows Terminal 사용자 지정 프로필 {#windows_terminal}

**검사 ID:** `windows_terminal` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 패키지형, 미리 보기 및 비패키지형 Windows Terminal JSON/JSONC 설정을 분석하여, 비어 있지 않은 프로필/기본 `commandline` 설정을 보고합니다.

**검사하는 증거:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` 및 WindowsTerminalPreview의 동등한 파일.
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: 레거시 profiles 배열, `profiles.list` 및 `profiles.defaults`의 commandline 필드.

**조사 방법:** 일반적인 셸은 당연히 여기에 나타납니다. 선택된/기본 프로필을 확인하고 비정상적인 명령이나 실행 인수를 확인하십시오. 프로필을 저장하는 것만으로는 실행되지 않습니다.

### WMI 이벤트 구독 {#wmi_event_subscription}

**검사 ID:** `wmi_event_subscription` · **카탈로그 ATT&CK:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**Juntei가 검사하는 내용:** 수집된 WMI OBJECTS.DATA 저장소에서 인식된 이벤트 소비자(consumer) 인스턴스와 필터-소비자 바인딩 참조를 카빙합니다. 인식되는 클래스에는 명령줄, 액티브 스크립트, 로그 파일, 이벤트 로그 및 SMTP 소비자가 포함됩니다.

**검사하는 증거:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` 및 `Repository\FS\OBJECTS.DATA`.

**조사 방법:** 원시 저장소에서 발견된 항목은 오래되었거나, 바인딩되지 않았거나, 과거의 것일 수 있으며, 출력은 이러한 단서를 보존합니다. 카빙은 완전한 활성 구독 그래프를 재구성하지 않으며 트리거가 발동했음을 증명하지도 않습니다. 저장소 파일의 쓰기 시간은 구독 생성/실행 시간이 아닙니다.

## 탐색기, COM 및 파일 또는 URL 처리기 { #explorer-com-and-file-or-url-handlers }

COM(Component Object Model)을 통해 애플리케이션은 등록된 소프트웨어 구성 요소를 활성화할 수 있습니다. Windows 셸은 등록된 처리기를 사용하여 파일, URL, 폴더 및 특수 개체를 엽니다. COM 식별자(CLSID)는 구성 요소를 식별하며, ProgID는 일부 구성 요소에 읽기 쉬운 이름을 부여합니다. 각 등록을 따라 그것이 선택하는 명령이나 DLL까지 추적하십시오.

### App Paths 하이재킹 {#app_paths}

**검사 ID:** `app_paths` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 셸 기반 실행 시 애플리케이션 조회를 리디렉션할 수 있는 시스템 전체 `App Paths` 등록 아래의 기본값을 읽습니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`의 기본값.

**조사 방법:** App Paths 등록은 설치된 애플리케이션에서 흔히 볼 수 있으며, 모든 프로세스 실행을 일반적으로 다시 쓰는 것이 아닙니다. 등록된 이름과 이미지를 예상되는 소프트웨어와 비교한 다음, 관련 셸/프로세스 증거를 검토하십시오.

### COM 개체 하이재킹 {#com_treat_as}

**검사 ID:** `com_treat_as` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 지원되는 네이티브/32비트 클래스 저장소 전반에서 COM `TreatAs` 리디렉션과, 시스템 CLSID를 가리는 사용자별 InprocServer32/LocalServer32 등록을 찾습니다. TreatAs 대상은 같은 사용자의 클래스와 시스템 클래스를 사용하여 확인합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` 및 시스템 `Classes\Wow6432Node` 뷰.
- NTUSER.DAT의 사용자별 `Software\Classes\CLSID`와 UsrClass.dat의 `CLSID`(구현된 `Wow6432Node` 뷰 포함): `TreatAs`, `InprocServer32` 및 `LocalServer32`의 기본값.

**조사 방법:** 사용자별 재정의에는 관리자 권한이 필요하지 않을 수 있지만, 정상 소프트웨어도 이를 등록할 수 있습니다. 선택된 클래스와 DLL/실행 파일을 추적하고, 아키텍처와 활성화 컨텍스트를 평가하며, 실제 COM 활성화는 독립적으로 확증하십시오.

### 디스크 정리 처리기 하이재킹 {#disk_cleanup_handler}

**검사 ID:** `disk_cleanup_handler` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 디스크 정리 VolumeCaches 처리기 CLSID를 읽고, 확인 가능한 시스템 COM InprocServer32 등록을 따라 해당 라이브러리까지 추적합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>`의 기본 CLSID 및 `Classes\CLSID\<CLSID>\InprocServer32`.

**조사 방법:** 처리기가 실행되려면 관련 정리 작업 흐름이 이를 호출해야 합니다. 이 검사는 확인되지 않은 처리기를 출력하지 않으므로, 수집본이 불완전하다면 레지스트리 파서를 사용하여 원시 등록 정보를 확인하십시오.

### Explorer AppKey 재정의 {#explorer_app_key}

**검사 ID:** `explorer_app_key` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Explorer `AppKey` 바로 아래 하위 항목의 `ShellExecute` 및 `Association` 값을 읽어, 애플리케이션 키로 선택되는 명령/동작을 다룹니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` 및 `Association`.

**조사 방법:** 키보드 유틸리티와 OEM 도구가 이 값을 정의할 수 있습니다. 연결(association) 이름은 파일 경로가 아니라 또 다른 레지스트리 피벗이 필요할 수 있습니다. 이를 트리거할 사용자 동작을 확인하십시오.

### 브라우저 도우미 개체(BHO) {#explorer_bho}

**검사 ID:** `explorer_bho` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Browser Helper Object CLSID 등록을 읽고, 가능한 경우 시스템 InprocServer32 라이브러리를 확인합니다. 확인되지 않은 클래스 ID도 계속 표시됩니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` 및 시스템 `Classes\CLSID\<CLSID>\InprocServer32`의 기본값.

**조사 방법:** 이는 호환되는 브라우저/Explorer 동작과 연관되며, 특히 레거시 환경과 관련이 있습니다. 승인된 브라우저/보안 확장이 나타날 수 있습니다. 등록된 BHO는 현재 브라우저가 이를 로드했다는 증거가 아닙니다.

### Explorer 데스크톱 CLSID 하이재킹 {#explorer_clsid_hijack}

**검사 ID:** `explorer_clsid_hijack` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 시스템/사용자 클래스 저장소에서, 나열된 세 개의 특수 Explorer 네임스페이스 CLSID에 구성된 셸 열기 명령을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**조사 방법:** 바탕 화면, 내 컴퓨터 및 내 문서의 명령 처리기를 일치하는 기준선과 비교하십시오. 이 검사는 선택된 CLSID와 경로만 검사하며, 가능한 모든 Explorer 네임스페이스 하이재킹을 검사하지는 않습니다.

### Explorer 컨텍스트 메뉴 처리기 {#explorer_context_menu}

**검사 ID:** `explorer_context_menu` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** Directory, Directory Background, Folder, Drive, AllFilesystemObjects 및 `*`에 대한 컨텍스트 메뉴 처리기를 열거하며, 가능한 경우 클래스 ID를 따라 수집된 DLL 등록까지 추적합니다.

**검사하는 증거:**

- 시스템 `HKLM\SOFTWARE\Classes` 및 사용자 UsrClass.dat 클래스 저장소: `<subject>\shellex\ContextMenuHandlers\<handler>`의 기본값과 이에 대응하는 COM 라이브러리 등록.

**조사 방법:** 압축 도구, 보안 제품, 클라우드 클라이언트의 오른쪽 클릭 확장은 흔합니다. 실제 처리기와 이를 트리거하는 상호 작용을 확인하십시오. 확인되지 않은 GUID는 DLL이 존재하지 않았다는 증거가 아니라 여전히 유용한 단서입니다.

### 파일 연결 하이재킹 {#file_association_hijack}

**검사 ID:** `file_association_hijack` · **카탈로그 ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei가 검사하는 내용:** `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` 및 `.ps1`에 대해 확장자에 직접 등록된 항목과 확인된 ProgID의 `shell\open\command` 등록을 검사합니다.

**검사하는 증거:**

- 시스템 `HKLM\SOFTWARE\Classes` 및 사용자 UsrClass.dat 클래스 저장소: 선택된 `<extension>`의 기본값과 `<extension-or-ProgID>\shell\open\command`의 기본값.

**조사 방법:** 예상되는 애플리케이션과 비교하고 명령 인수를 확인하십시오. 이는 정해진 확장자 집합이며, 모든 파일 형식이나 UserChoice를 포함한 완전한 유효 확인 결과가 아닙니다. 저장된 명령은 문서가 열렸다는 증거가 아닙니다.

### 프로토콜 처리기 하이재킹 {#protocol_handler_hijack}

**검사 ID:** `protocol_handler_hijack` · **카탈로그 ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei가 검사하는 내용:** http, https, mailto, ms-msdt 및 ms-officecmd의 URL 열기 명령과, 발견된 `URL Protocol` 등록 및 해당하는 사용자 재정의를 읽습니다.

**검사하는 증거:**

- 시스템 `HKLM\SOFTWARE\Classes` 및 사용자 UsrClass.dat 클래스 저장소: `URL Protocol` 표시와 `<scheme>\shell\open\command`의 기본값.

**조사 방법:** 브라우저와 애플리케이션 설치 프로그램은 정상적으로 스킴을 등록합니다. 명령과 인수를 추적하고, 스킴의 예상 용도를 평가하며, URL 열기/프로세스 증거와 상관 분석하십시오.

### 휴지통 COM 확장 처리기 {#recycle_bin_com_extension}

**검사 ID:** `recycle_bin_com_extension` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 나열된 휴지통 CLSID의 open/empty/explore 명령 재정의와 바로 아래의 컨텍스트 메뉴/끌어서 놓기 처리기 등록을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**조사 방법:** 승인된 셸 확장과 비교하십시오. 보고된 일부 값은 직접 확인된 라이브러리가 아니라 COM 식별자입니다. 코드 실행을 주장하기 전에 참조된 클래스와 사용자 동작을 조사하십시오.

### 검색 프로토콜 처리기 하이재킹 {#search_protocol_handler}

**검사 ID:** `search_protocol_handler` · **카탈로그 ATT&CK:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**Juntei가 검사하는 내용:** 시스템/사용자 `search-ms\shell\open\command`의 기본 처리기를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**조사 방법:** 처리기가 변경되면 search-ms URL을 열 때의 동작이 바뀔 수 있지만, 그 존재가 익스플로잇 전달의 증거는 아닙니다. Windows 기본값과 비교하고, 이를 사용하게 될 URL/프로세스 활동을 조사하십시오.

### SharedTaskScheduler {#shared_task_scheduler}

**검사 ID:** `shared_task_scheduler` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 비어 있지 않은 Explorer `SharedTaskScheduler` 값을 등록된 셸 구성 요소로 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | 비어 있지 않은 모든 값 |

**조사 방법:** 값은 실행 파일 경로가 아니라 COM 클래스를 참조할 수 있습니다. 해당 서버 등록을 추적하고 운영 체제 동작, 확장의 출처 및 관련 셸 로드 증거를 확인하십시오.

### ShellExecuteHooks {#shell_execute_hooks}

**검사 ID:** `shell_execute_hooks` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 비어 있지 않은 Explorer `ShellExecuteHooks` 값을 열거하여 레거시 셸 실행 후크 등록을 다룹니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | 비어 있지 않은 모든 값 |

**조사 방법:** 참조된 CLSID를 확인하고 Windows 빌드와 정책이 해당 후크를 허용하는지 검증하십시오. 레거시 등록은 사용되지 않을 수 있으며, 값 자체는 관찰된 셸 실행이 아닙니다.

### 시작 메뉴 라이브러리 및 CLSID 폴더 COM 지속성 {#start_menu_com}

**검사 ID:** `start_menu_com` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` 아래에서
`simpleLocation` URL이 COM CLSID를 참조하는 `.library-ms` 라이브러리 XML 파일과
`<name>.{CLSID}` 형식의 이름을 가진 디렉터리를 찾습니다. Explorer가 이 개체와 상호 작용하면
COM 코드가 활성화될 수 있습니다. 이 검사는 수집된 빈 디렉터리도 포함하며, 심볼릭 링크는 따라가지
않습니다. 시작 메뉴 외부의 일반 라이브러리 파일과 XML 주석 안의 GUID는 이 검사에서 보고되지
않습니다.

**검사하는 증거:**

- 사용자별 시작 메뉴의 `.library-ms` 파일과 CLSID 이름을 가진 디렉터리 항목.
- 일치하는 사용자의 수집된 클래스 등록과 시스템 SOFTWARE(아키텍처별로 구분).
- 예약된 작업과 같은 확인 로직을 통한 관리 코드 CodeBase/AppID/서비스 구성.

**조사 방법:** `Path`와 `SourceFile`은 활성화 트리거 아티팩트를 식별합니다. `Value`에는
후보 서버 명령이 들어 있으며, 확인할 수 없는 경우에는 CLSID가 들어 있습니다. `Description`은
트리거를 CLSID 및 수집된 레지스트리 원본과 연결합니다. `ResolvedPath`/`Sha256`은 확인되고
수집된 페이로드만 나타냅니다. 실행 파일 이름만 있는 AppID 연결은 확인되지 않은 상태로 남습니다.
트리거는 정상일 수 있으며, 존재만으로는 로그인, 활성화, 실행 또는 공격이 증명되지 않습니다.
라이브러리 URL, 클래스 등록, 페이로드, 사용자 컨텍스트 및 독립적인 실행 로그를 확인하십시오.

**수집 시 주의 사항:** 파일만 수집하면 빈 디렉터리가 누락될 수 있습니다. 표준 사용자 시작 메뉴
경로만 검사하며, 리디렉션된 시작 메뉴와 기타 Explorer 네임스페이스 위치는 이 검사에서 다루지
않습니다. 하이브가 없거나 등록이 확인되지 않으면 후보는 유지한 채 부분 커버리지로 처리됩니다.
작업 아키텍처, 권한 상승 및 정책에 따라 실제 COM 조회 동작이 달라질 수 있습니다.

**추가 정보:** [최초 제안 및 연구자 크레딧](https://github.com/last-byte/PersistenceSniper/pull/32),
[COM 사용자별/시스템 병합 뷰](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM 하이재킹](https://attack.mitre.org/techniques/T1546/015/).

### TypeLib COM 하이재킹 {#typelib_hijack}

**검사 ID:** `typelib_hijack` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 사용자별 UsrClass.dat의 TypeLib GUID/버전/로캘 win32/win64 등록을 읽고, 경로와 스크립트 모니커 값을 유지합니다. 끝에 붙은 숫자 리소스 인덱스는 대상 파일을 확인할 때만 제거합니다.

**검사하는 증거:**

- 수집된 각 UsrClass.dat의 `TypeLib\<GUID>\<version>\<locale>\win32` 및 `win64` 기본값.

**조사 방법:** 형식 라이브러리는 정상 DLL, 다른 파일 또는 스크립트 모니커를 가리킬 수 있습니다. 이 검사는 이러한 등록의 목록을 작성합니다. 기본값이 아닌 모든 경로를 공격으로 가정하지 말고, 영향을 받는 애플리케이션을 평가하고 형식 라이브러리 로드를 확증하십시오.

## Office 애플리케이션 및 추가 기능 { #office-applications-and-add-ins }

Office는 정상적인 추가 기능, 템플릿 및 애플리케이션별 확장을 지원합니다. 의심스러운 등록이 실제로 의미를 가지는지는 특정 Office 버전, 애플리케이션, 로드 설정 또는 사용자 상호 작용에 따라 달라질 수 있습니다. 구성만으로는 매크로 실행이 입증되지 않습니다.

### Office 추가 기능 등록 {#office_addins}

**검사 ID:** `office_addins` · **카탈로그 ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei가 검사하는 내용:** 구현된 시스템 Click-to-Run 저장소와 사용자 루트를 포함하여, 버전 없는 Office 루트와 14.0/15.0/16.0 Office 루트에서 Word, Excel, PowerPoint, Outlook 및 Access 추가 기능을 열거합니다. 명시적인 Manifest/FileName/Path 값을 유지하거나 조건에 맞는 COM 기반 항목을 확인하며, LoadBehavior 컨텍스트를 함께 제공합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` 및 구현된 시스템 Click-to-Run의 동등한 위치.
- NTUSER.DAT의 `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>`. SOFTWARE, NTUSER.DAT 및 UsrClass.dat에서 일치하는 시스템/사용자 COM 클래스.

**조사 방법:** 추가 기능은 흔한 업무용 소프트웨어입니다. LoadBehavior, 애플리케이션/버전, 매니페스트, 실제 페이로드 및 예상 공급업체를 검토하십시오. 이 검사는 설치된 모든 추가 기능 형식이나 등록 뷰가 수집되었거나 로드되었음을 보장하지 않습니다.

### Office AI 추가 기능 하이재킹 {#office_ai_hijack}

**검사 ID:** `office_ai_hijack` · **카탈로그 ATT&CK:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**Juntei가 검사하는 내용:** Office 16.0 Click-to-Run의 Common AI 등록 위치에 있는 값을 열거합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: 모든 값.

**조사 방법:** 비정상적인 값과 경로를 설치된 Office 빌드와 비교하십시오. 값이 존재한다고 해서 어떤 Office 기능이 이를 사용했는지 알 수 있거나, 실행 코드를 선택했다는 것이 증명되지는 않습니다.

### Office DLL 재정의 {#office_dll_override}

**검사 ID:** `office_dll_override` · **카탈로그 ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei가 검사하는 내용:** 구현된 Click-to-Run 및 사용자 루트를 포함하여, 버전 없는 설정과 14.0/15.0/16.0 설정의 Word/PowerPoint 루트 아래에서 `WwlibtDll` 및 `PPCoreTDLL`을 읽습니다.

**검사하는 증거:**

- 시스템/사용자 `Microsoft\Office\[<version>\]Word` 및 `PowerPoint` 설정과 구현된 시스템 Click-to-Run Office 트리: `WwlibtDll` 및 `PPCoreTDLL`.

**조사 방법:** 어떤 애플리케이션/빌드가 이 재정의를 인식하는지 확인하고, 라이브러리와 출처를 확인한 다음, Office 실행 또는 이미지 로드와 상관 분석하십시오. 구성된 재정의만으로는 실행의 증거가 되지 않습니다.

### Office 기본 템플릿 {#office_templates}

**검사 ID:** `office_templates` · **카탈로그 ATT&CK:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**Juntei가 검사하는 내용:** 수집된 Normal.dotm/NormalEmail.dotm 파일과, Office 설치 시작 폴더를 포함하여 인식된 Word Startup 및 Excel XLSTART 폴더 바로 아래의 파일 목록을 작성합니다.

**검사하는 증거:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` 및 `NormalEmail.dotm`.
- Roaming AppData 아래 사용자 `Microsoft\Word\STARTUP` 및 `Microsoft\Excel\XLSTART` 바로 아래의 파일.
- Program Files 및 Program Files (x86) 아래 인식된 Office `<version>\STARTUP` / `XLSTART` 및 `root\<version>\STARTUP` / `XLSTART` 폴더 바로 아래의 파일.

**조사 방법:** 템플릿은 정상일 수도 있고 실행 가능한 콘텐츠가 전혀 없을 수도 있습니다. 이 검사에서 Juntei는 매크로를 추출하거나 실행하지 않습니다. 문서 내부 구조, 매크로 정책 및 애플리케이션 활동을 확인하십시오. 시스템 설치 경로라고 해서 SYSTEM 프로세스 토큰을 의미하지는 않습니다.

### Office Test DLL {#office_test_dll}

**검사 ID:** `office_test_dll` · **카탈로그 ATT&CK:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**Juntei가 검사하는 내용:** 나열된 시스템/사용자 뷰와 32비트 시스템 뷰에서 Office Test Special Perf 키의 값을 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | 비어 있지 않은 모든 값 |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | 비어 있지 않은 모든 값 |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | 비어 있지 않은 모든 값 |

**조사 방법:** 예상하지 못한 재정의는 Office 버전 및 페이로드 검증이 필요한 단서로 취급하십시오. 이 검사는 구성을 보고할 뿐, 특정 Office 설치가 이 값을 로드하는지 테스트하지는 않습니다.

### Outlook 홈페이지 공격 {#outlook_home_page}

**검사 ID:** `outlook_home_page` · **카탈로그 ATT&CK:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**Juntei가 검사하는 내용:** Office 버전 14.0, 15.0 및 16.0에 대해 사용자 Outlook WebView 하위 항목 아래의 `URL`을 읽습니다.

**검사하는 증거:**

- NTUSER.DAT의 `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL`.

**조사 방법:** 폴더 홈페이지는 정상일 수 있으며, 동작은 Outlook 버전과 보안 업데이트에 따라 다릅니다. URL/콘텐츠와 유효 정책을 확인한 다음, Outlook/웹/프로세스 증거와 상관 분석하십시오.

### VBA Monitor DLL 하이재킹 {#vba_monitors}

**검사 ID:** `vba_monitors` · **카탈로그 ATT&CK:** [T1137](https://attack.mitre.org/techniques/T1137/)

**Juntei가 검사하는 내용:** 시스템 및 사용자별 클래스 저장소에서, 구현된 VBA 모니터 관련 CLSID 두 개의 InprocServer32 값을 읽습니다.

**검사하는 증거:**

- 시스템 SOFTWARE의 `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` 및 `{0002E157-0000-0000-C000-000000000046}`, 그리고 사용자 UsrClass.dat의 동등한 등록.

**조사 방법:** 선택된 라이브러리를 설치된 Office/VBA 환경과 비교하십시오. 이는 특정 CLSID를 대상으로 한 검사이며, 모든 VBA 확장의 목록이나 매크로가 실행되었다는 증거가 아닙니다.

## 디버거, 크래시 처리기 및 호환성 shim { #debuggers-crash-handlers-and-compatibility-shims }

디버거나 오류 처리기는 애플리케이션이 시작, 크래시, 응답 없음 또는 종료될 때 실행될 수 있습니다. 애플리케이션 호환성 데이터베이스도 시작에 영향을 줄 수 있습니다. 보고된 명령과 함께 전제 조건이 되는 설정과 이를 트리거하는 프로그램을 조사하십시오.

### AeDebug 디버거 하이재킹 {#ae_debug}

**검사 ID:** `ae_debug` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** JIT(Just-In-Time) 크래시 디버깅 구성에 사용되는 네이티브 및 32비트 AeDebug `Debugger` 명령을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**조사 방법:** 개발/디버깅 소프트웨어가 정상적으로 이 값을 설정할 수 있습니다. 전체 디버거 명령과 크래시 트리거 전제 조건을 검토한 다음, 애플리케이션 오류 및 프로세스 이벤트와 상관 분석하십시오.

### AeDebug 보호된 프로세스 디버거 {#ae_debug_protected}

**검사 ID:** `ae_debug_protected` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** 보호된 프로세스 디버깅 영역에 대한 네이티브 및 32비트 AeDebugProtected `Debugger` 명령을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**조사 방법:** 운영 체제 빌드와 보호 정책이 구성된 명령을 사용할 수 있는지 확인하십시오. 저장된 디버거는 보호된 프로세스가 크래시되었거나 명령이 실행되었다는 증거가 아닙니다.

### Image File Execution Options 디버거 {#ifeo_debugger}

**검사 ID:** `ifeo_debugger` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** 중첩된 필터 키를 포함하여 네이티브/32비트 Image File Execution Options 등록에서 `Debugger` 값을 검색합니다.

**검사하는 증거:**

- 중첩된 필터 키와 시스템 `Wow6432Node` 뷰를 포함한 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger`.

**조사 방법:** 지정된 이미지가 실행될 때 디버거가 선택될 수 있습니다. 이미지/필터 범위와 디버거 인수를 확인하십시오. 개발자 도구도 IFEO를 사용할 수 있습니다. 실행을 입증하려면 대상의 실행을 추적하십시오.

### IFEO 위임 NTDLL {#ifeo_delegated_ntdll}

**검사 ID:** `ifeo_delegated_ntdll` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** 중첩된 키를 포함하여 네이티브/32비트 Image File Execution Options 등록에서 `VerifierDlls` 재정의를 검색합니다.

**검사하는 증거:**

- 중첩된 키와 시스템 `Wow6432Node` 뷰를 포함한 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls`.

**조사 방법:** 이 검사의 판단 조건은 구성된 VerifierDlls 값이며, 모든 검증기/위임 전제 조건이 활성화되어 있음을 증명하지 않습니다. 작동하는 인젝션 경로를 추론하기 전에 해당 플래그, Windows 버전 및 페이로드를 검토하십시오.

### 자동 프로세스 종료 모니터(Silent Process Exit) {#ifeo_silent_process_exit}

**검사 ID:** `ifeo_silent_process_exit` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** 중첩된 키를 포함하여 네이티브/32비트 SilentProcessExit 등록에서 `MonitorProcess`를 검색합니다.

**검사하는 증거:**

- 중첩된 키와 시스템 `Wow6432Node` 뷰를 포함한 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess`.

**조사 방법:** 자동 종료 모니터링에는 관련 보고/IFEO 구성도 필요합니다. 해당 설정과 모니터링 대상 이미지의 종료 이벤트를 확인하십시오. 정상적인 진단 도구로 모니터가 설명될 수 있습니다.

### 애플리케이션 shim {#installed_sdb}

**검사 ID:** `installed_sdb` · **카탈로그 ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei가 검사하는 내용:** 설치된 애플리케이션 호환성 데이터베이스에 대해, InstalledSDB 바로 아래의 등록에서 `DatabasePath`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**조사 방법:** 호환성 수정은 정상일 수 있습니다. SDB 내용, 대상 실행 파일 및 설치 이력을 확인하십시오. 데이터베이스 등록은 그 안의 shim을 식별하지 않으며, 일치하는 애플리케이션이 실행되었음을 증명하지도 않습니다.

### LSM 디버거 하이재킹 {#lsm_debugger}

**검사 ID:** `lsm_debugger` · **카탈로그 ATT&CK:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**Juntei가 검사하는 내용:** 네이티브 및 32비트 시스템 뷰에서 `lsm.exe`에 대한 SilentProcessExit 아래의 `MonitorProcess`만을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**조사 방법:** 이는 더 넓은 SilentProcessExit 목록과 중복됩니다. 구성을 실행된 지속성으로 취급하기 전에 전제 조건 플래그와 관련 LSM 종료를 확인하십시오. 서로 다른 결과를 평가할 때는 같은 항목을 한 번만 세십시오.

### MiniDump 보조 DLL {#minidump_auxiliary_dlls}

**검사 ID:** `minidump_auxiliary_dlls` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** MiniDumpAuxiliaryDlls에서 이름이 있고 비어 있지 않은 값을 열거하며, 값 이름은 트리거 모듈 컨텍스트로, 값 데이터는 선택된 대상으로 보존합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: 트리거 모듈 값 이름과 대상 데이터.

**조사 방법:** 트리거 모듈과 보조 DLL을 모두 확인하십시오. 크래시/미니덤프 도구로 항목이 설명될 수 있으며, 등록 정보는 덤프 작업이 라이브러리를 로드했음을 보여 주지 않습니다.

### 애플리케이션 shim {#shim_custom}

**검사 ID:** `shim_custom` · **카탈로그 ATT&CK:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**Juntei가 검사하는 내용:** AppCompatFlags Custom 바로 아래의 애플리케이션 키에서 `.sdb`라는 이름을 가진 값을 읽고, 애플리케이션과 데이터베이스의 연결을 보존합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: 이름이 `.sdb`로 끝나는 값.

**조사 방법:** 이러한 연결을 InstalledSDB 및 실제 호환성 데이터베이스와 함께 확인하십시오. 파일 이름 연결만으로는 데이터베이스의 동작을 알 수 없으며, 일치하는 애플리케이션이 실행되었음을 입증하지도 않습니다.

### WER 디버거 하이재킹 {#wer_debugger}

**검사 ID:** `wer_debugger` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 네이티브/32비트 Windows 오류 보고(Windows Error Reporting)의 `Debugger` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**조사 방법:** 정상적인 진단 소프트웨어가 디버거를 구성할 수 있습니다. 전체 명령과 관련 크래시 이벤트를 조사하십시오. 등록 정보는 Windows 오류 보고가 이를 시작했다는 증거가 아닙니다.

### WER Hangs 디버거 하이재킹 {#wer_hangs}

**검사 ID:** `wer_hangs` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 네이티브/32비트 Windows 오류 보고 Hangs의 `Debugger` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**조사 방법:** 응답 없음 처리 작업 흐름이 이 설정을 호출하는지 확인하고 대상 프로그램을 확인하십시오. 레지스트리 쓰기를 디버거 실행으로 간주하지 말고, 애플리케이션 응답 없음 및 프로세스 기록과 상관 분석하십시오.

### WER ReflectDebugger 하이재킹 {#wer_reflect_debugger}

**검사 ID:** `wer_reflect_debugger` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 네이티브/32비트 Windows 오류 보고의 `ReflectDebugger` 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**조사 방법:** 승인된 진단 구성과 비교하고 Windows 버전/트리거 적용 여부를 확인하십시오. 실행으로 판단하기 전에 명령 인수를 확인하고 크래시/프로세스를 통한 확증을 찾으십시오.

### WER 런타임 예외 모듈 {#wer_runtime_exception}

**검사 ID:** `wer_runtime_exception` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 네이티브/32비트 RuntimeExceptionHelperModules 아래의 값 이름을 열거합니다. 여기서는 라이브러리 경로가 값의 데이터가 아니라 값 이름에 들어 있습니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` 및 시스템 `Wow6432Node` 뷰: 값 이름.

**조사 방법:** 지정된 모듈과 애플리케이션/런타임 컨텍스트를 확인하십시오. 데이터에 들어 있는 작은 숫자 값은 페이로드 경로가 아니며, 등록만으로는 도우미가 로드되었음을 증명하지 못합니다.

## .NET 런타임 및 프로파일러 설정 { #net-runtime-and-profiler-settings }

관리형 런타임은 애플리케이션 코드보다 먼저 프로파일러, 시작 어셈블리 또는 디버깅 라이브러리를 로드할 수 있습니다. 이 설정들은 개발자와 모니터링 제품도 사용합니다. 런타임, 프로세스 아키텍처, 활성화 플래그, 그리고 새로 시작된 프로세스가 이 설정을 상속할 수 있는지 확인하십시오.

### AppDomainManager 인젝션 {#appdomain_manager}

**검사 ID:** `appdomain_manager` · **카탈로그 ATT&CK:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**Juntei가 검사하는 내용:** 사용자 지정 .NET AppDomain 관리자의 어셈블리 및 형식 선택을 다루는 시스템/사용자 `APPDOMAIN_MANAGER_ASM` 및 `APPDOMAIN_MANAGER_TYPE` 환경 변수를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**조사 방법:** 애플리케이션이 의도적으로 사용자 지정 관리자를 사용할 수 있습니다. 어셈블리와 형식을 함께 확인하고 런타임 호환성과 프로세스 상속을 점검하십시오. 단일 변수나 그 키의 타임스탬프만으로는 관리형 애플리케이션이 이를 로드했다는 것이 증명되지 않습니다.

### .NET CLR 프로파일러 하이재킹 {#cor_profiler}

**검사 ID:** `cor_profiler` · **카탈로그 ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei가 검사하는 내용:** 시스템/사용자 COR 프로파일링 환경 설정과 개별 서비스의 Environment 다중 문자열에서 이름이 일치하는 항목을 읽습니다. 활성화 플래그, 프로파일러 식별자 및 아키텍처별 경로를 유지합니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` 및 수집된 각 사용자 NTUSER.DAT의 `Environment`.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: 다중 문자열 안의 이름이 지정된 COR 설정.

**조사 방법:** 프로파일러는 정상적인 모니터링을 지원합니다. 활성화 플래그, CLSID 및 DLL 경로는 각각 별개의 구성 요소입니다. 각 행을 실행된 프로파일러로 취급하지 말고, 실제 활성화 여부와 관련 .NET 프로세스를 확인하십시오.

### .NET Core CLR 프로파일러 하이재킹 {#coreclr_profiler}

**검사 ID:** `coreclr_profiler` · **카탈로그 ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei가 검사하는 내용:** 아키텍처별 경로를 포함하여, 이에 해당하는 시스템/사용자 CORECLR 프로파일링 환경 설정과 일치하는 서비스 Environment 항목을 읽습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` 및 수집된 각 사용자 NTUSER.DAT의 `Environment`.
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: 다중 문자열 안의 이름이 지정된 CORECLR 설정.

**조사 방법:** .NET 런타임, 아키텍처, 활성화 플래그 및 라이브러리를 검증하십시오. 승인된 관측성(observability) 제품이 이 설정을 구성할 수 있으며, 기존 프로세스는 새로 변경된 환경을 상속하지 않을 수 있습니다.

### .NET DbgManagedDebugger 하이재킹 {#dotnet_dbg_managed_debugger}

**검사 ID:** `dotnet_dbg_managed_debugger` · **카탈로그 ATT&CK:** [T1546](https://attack.mitre.org/techniques/T1546/)

**Juntei가 검사하는 내용:** 명시적인 시스템 32비트 경로를 포함하여, 나열된 시스템/사용자 `.NETFramework`의 `DbgManagedDebugger` 설정을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**조사 방법:** 해당 .NET 런타임/디버깅 정책이 이 명령을 사용하는지 확인하십시오. 개발자 도구로 설명될 수 있습니다. 실행을 입증하려면 관리 코드 크래시 및 프로세스 생성과 상관 분석하십시오.

### .NET Framework COR_PROFILER 레지스트리 키 {#dotnet_framework_profiler}

**검사 ID:** `dotnet_framework_profiler` · **카탈로그 ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei가 검사하는 내용:** 나열된 `.NETFramework` 레지스트리 뷰에서 `COR_PROFILER`, `COR_PROFILER_PATH` 및 `COR_ENABLE_PROFILING`을 읽습니다. 이는 환경 변수 기반 프로파일러 검사를 보완합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**조사 방법:** 값들을 함께 평가하고, 런타임이 해당 레지스트리 위치를 따르는지 확인하십시오. 구성된 프로파일링 도구는 정상일 수 있습니다. 단독 활성화 플래그를 식별된 DLL 로드로 해석하지 마십시오.

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**검사 ID:** `dotnet_startup_hooks` · **카탈로그 ATT&CK:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**Juntei가 검사하는 내용:** 지원되는 .NET 런타임에서 시작 후크 어셈블리를 선택할 수 있는 시스템/사용자 `DOTNET_STARTUP_HOOKS` 환경 값을 읽습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` 및 각 사용자 NTUSER.DAT의 `Environment\DOTNET_STARTUP_HOOKS`.

**조사 방법:** 여러 어셈블리가 있는지 전체 값을 확인하고 런타임 지원 여부를 검증하십시오. 이 검사는 서비스별 시작 후크 환경을 열거하지 않으며, 어떤 프로세스가 이 값을 상속하여 사용했음을 증명하지도 않습니다.

### 알려진 관리 디버깅 DLL {#known_managed_debugging_dlls}

**검사 ID:** `known_managed_debugging_dlls` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** `.NETFramework`의 `KnownManagedDebuggingDlls` 등록을 관리 디버거 라이브러리 재정의 후보로 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**조사 방법:** 설치된 개발/런타임 도구와 비교하고 이를 트리거하는 디버거를 확인하십시오. 레지스트리 항목은 디버거 세션이나 라이브러리 로드를 증명하지 않습니다.

## DLL 및 실행 파일 조회 재정의 { #dll-and-executable-lookup-overrides }

이 검사들은 구성 요소가 코드를 찾는 방식에 영향을 주는 명시적인 라이브러리 위치나 설정을 검사합니다. 경로 재정의나 약화된 검색 정책은 조사의 단서입니다. 실행이 이루어지려면 여전히 페이로드와 호환되는 트리거 구성 요소가 필요합니다.

### AppCert DLLs {#appcert_dlls}

**검사 ID:** `appcert_dlls` · **카탈로그 ATT&CK:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**Juntei가 검사하는 내용:** 프로세스 생성 API와 연관된 라이브러리 확장 영역인 Session Manager AppCertDlls의 비어 있지 않은 등록을 열거합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | 비어 있지 않은 모든 값 |

**조사 방법:** 각 라이브러리와 그 설치 출처를 확인하십시오. 적용 여부와 런타임 로드는 별도로 확인하십시오. 타임라인에 일반적인 프로세스 생성이 있다는 것만으로는 구성된 DLL이 로드되었다는 증거가 되지 않습니다.

### AppInit DLLs {#appinit_dlls}

**검사 ID:** `appinit_dlls` · **카탈로그 ATT&CK:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**Juntei가 검사하는 내용:** 네이티브/32비트 `AppInit_DLLs`를 읽고 라이브러리 목록을 분리한 다음, 사용 가능한 `LoadAppInit_DLLs` 및 `RequireSignedAppInit_DLLs` 컨텍스트를 각 후보에 추가합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` 및 `RequireSignedAppInit_DLLs`. 시스템 `Wow6432Node` 뷰도 포함합니다.

**조사 방법:** LoadAppInit_DLLs 플래그가 0이면 명시적으로 비활성 상태로 표시됩니다. Windows 버전, 보안 부팅(Secure Boot) 및 서명 요구 사항도 로드에 영향을 줍니다. AppInit DLL이 실행되었다고 주장하기 전에 유효 설정을 검토하십시오.

**추가 정보:** [Microsoft 문서](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls).

### AutodialDLL 재정의 {#autodial_dll}

**검사 ID:** `autodial_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** WinSock2 Parameters의 `AutodialDLL` 재정의를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**조사 방법:** 이 설정을 사용하는 네트워킹 기능과 원본 Windows 버전을 확인하십시오. 실제 DLL과 네트워크/프로세스 활동을 검토하십시오. 재정의만으로는 활성 로드나 악성 로드가 입증되지 않습니다.

### CHM 도우미 DLL {#chm_helper_dll}

**검사 ID:** `chm_helper_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 도움말 관련 라이브러리/위치 재정의 영역인 HtmlHelp Author 아래의 `Location`을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**조사 방법:** 이 시스템에서 관련 도움말 구성 요소가 이 값을 사용하는지 확인하고 선택된 파일을 확인하십시오. 레지스트리 재정의는 CHM 문서가 열렸다는 증거가 아닙니다.

### NL6 ContentIndex DLL 재정의 {#content_index_dll}

**검사 ID:** `content_index_dll` · **카탈로그 ATT&CK:** [T1574](https://attack.mitre.org/techniques/T1574/)

**Juntei가 검사하는 내용:** 활성 제어 집합의 ContentIndex Language 바로 아래 등록에서 `DLLOverridePath`를 읽습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`.

**조사 방법:** 언어/검색 구성 요소는 Windows 빌드마다 다를 수 있습니다. 라이브러리와 일치하는 설치된 구성 요소를 확인하고, 로드를 확증하려면 실제 인덱싱 프로세스를 추적하십시오.

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**검사 ID:** `crypto_expo_offload` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** Cryptography Offload의 `ExpoOffload` 값을 암호화 라이브러리 재정의 후보로 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**조사 방법:** 기능/빌드 적용 여부를 확인하고 선택된 라이브러리의 출처를 조사하십시오. 흔하지 않은 값이 있다는 것을 자격 증명 탈취나 코드 실행의 증거로 취급하지 마십시오.

### Direct3D 소프트웨어 래스터라이저 DLL {#direct3d_dll}

**검사 ID:** `direct3d_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** Direct3D Drivers의 `SoftwareRasterizer` 라이브러리 재정의를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**조사 방법:** 그래픽 및 호환성 소프트웨어가 래스터라이저 구성에 영향을 줄 수 있습니다. DLL과 호환되는 Direct3D 애플리케이션의 활동을 확인하십시오. 경로가 저장되어 있다고 해서 로드되었다는 것이 증명되지는 않습니다.

### DLL 검색 순서 약화 {#dll_search_mode}

**검사 ID:** `dll_search_mode` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 활성 Session Manager 값이 0으로 해석되는 경우에만 `SafeDllSearchMode` 또는 `CWDIllegalInDllSearch`를 보고합니다. 이는 검색 정책 조건이며, 선택된 페이로드가 아닙니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` 및 `CWDIllegalInDllSearch`.

**조사 방법:** 약화된 조회 정책은 별도의 검색 순서 공격을 더 쉽게 만들 수 있습니다. 이는 심어 놓은 DLL을 식별하거나 실행을 증명하지 않습니다. 0이 아니거나 없는 값은 이 검사에서 출력되지 않으므로, 유효 기본값과 정책은 별도로 평가하십시오.

**추가 정보:** [Microsoft 문서](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security).

### KnownDLLs 제외 {#exclude_from_known_dlls}

**검사 ID:** `exclude_from_known_dlls` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** Session Manager에서 비어 있지 않은 `ExcludeFromKnownDlls` 항목을 보호된/알려진 DLL 조회 동작의 변경으로 읽습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`.

**조사 방법:** 제외 항목은 조회 조건을 식별할 뿐, 대체 라이브러리 자체를 식별하지 않습니다. 영향을 받는 애플리케이션과 잠재적인 DLL 검색 위치를 찾은 다음, 비표준 이미지 로드를 확증하십시오.

**추가 정보:** [Microsoft 문서](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order).

### hhctrl.ocx DLL 재정의 {#hhctrl_ocx_dll}

**검사 ID:** `hhctrl_ocx_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 구현된 HTML 도움말 컨트롤 CLSID에 대해, 네이티브/32비트 시스템 및 지원되는 사용자 클래스 저장소의 InprocServer32 재정의를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**조사 방법:** 어떤 애플리케이션과 아키텍처가 이 컨트롤을 활성화하는지 확인하십시오. 등록된 DLL과 도움말 관련 활동을 확인하십시오. COM 등록은 HTML 도움말이 콘텐츠를 렌더링했다는 증거가 아닙니다.

### Known DLLs {#known_dlls}

**검사 ID:** `known_dlls` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 활성 Session Manager KnownDLLs 키에서 이름이 있는 값을 열거합니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: 이름이 있는 값.

**조사 방법:** 많은 항목은 예상되는 Windows 라이브러리이며, 일부 값은 페이로드 경로가 아니라 보조 구성입니다. 전체 키를 신뢰할 수 있는 같은 빌드와 비교하십시오. 이 검사는 아키텍처별 대체 KnownDLLs 트리를 모두 검사하지는 않습니다.

### LSA 확장 DLL {#lsa_extensions}

**검사 ID:** `lsa_extensions` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** LsaExtensionConfig LsaSrv 구성에서 `Extensions`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**조사 방법:** 보안 소프트웨어가 인증 구성 요소를 확장할 수 있습니다. 실제 라이브러리와 원본 Windows 버전과의 호환성을 확인한 다음, LSASS 이미지 로드 또는 기타 독립적인 사용 증거를 찾으십시오.

### MAPI32 DLL 경로 재정의 {#mapi32_dll_path}

**검사 ID:** `mapi32_dll_path` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 네이티브 및 32비트 시스템 뷰에서 Mail 클라이언트 바로 아래 등록의 `DLLPathEx` 및 `DLLPath`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**조사 방법:** 메일 클라이언트는 정상적인 MAPI 공급자를 등록합니다. 어떤 클라이언트/공급자가 선택되었는지 확인하고 실제 라이브러리를 확인하십시오. 설치된 공급자는 인시던트 기간 중 애플리케이션이 MAPI를 사용했다는 증거가 아닙니다.

### MSDTC XA DLL {#msdtc_xa_dll}

**검사 ID:** `msdtc_xa_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** MSDTC MTxOCI 구성에서 `OracleXaLib` 및 `OracleOciLib`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**조사 방법:** 데이터베이스 연동에 이 라이브러리가 정상적으로 필요할 수 있습니다. 구성된 경로가 실행되었다고 가정하지 말고, 트랜잭션 코디네이터 구성과 설치된 데이터베이스 소프트웨어를 검증하고 서비스/모듈 사용과 상관 분석하십시오.

### NLDP DLL 재정의 {#nldp_dll}

**검사 ID:** `nldp_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** NlsData의 `DllOverridePath` 구성 값을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**조사 방법:** 대상 라이브러리, Windows 빌드, 그리고 이 재정의를 사용할 수 있는 자연어 구성 요소를 검토하십시오. 이 값은 인덱싱이나 언어 처리가 발생했음을 보여 주지 않습니다.

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**검사 ID:** `rdp_test_dvc_plugin` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** 시스템 Terminal Server Client의 `TestDVCPlugin` 라이브러리 설정을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**조사 방법:** 익숙하지 않은 테스트 플러그인과 버전별 동작을 조사하십시오. 실행을 입증하려면 호환되는 클라이언트 세션과 라이브러리 로드가 필요하며, 레지스트리 값만으로는 어느 쪽도 알 수 없습니다.

### 검색 인덱서 DLL 재정의 {#search_indexer_dll}

**검사 ID:** `search_indexer_dll` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** SOFTWARE에서 Windows Search의 `DllPath` 재정의를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**조사 방법:** 일치하는 Search 구성 요소/버전을 확인하고 대상 파일을 확인하십시오. 비활성 상태인 Windows Search 서비스나 사용되지 않은 설정은 관찰된 악성 라이브러리 로드와 다릅니다.

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**검사 ID:** `winsock_auto_proxy` · **카탈로그 ATT&CK:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**Juntei가 검사하는 내용:** NameSpace_Catalog5의 Catalog_Entries 및 Catalog_Entries64 바로 아래 등록에서 `LibraryPath`를 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**조사 방법:** 네트워킹 공급자는 정상일 수 있습니다. 두 아키텍처, 공급자의 출처, 관련 네트워크 애플리케이션/이미지 로드 증거를 검토하십시오. 이 검사는 네임스페이스 공급자만을 대상으로 한 조회이며, 가능한 모든 Winsock 확장을 다루지는 않습니다.

## 계정 및 로그인 표시 여부 { #accounts-and-sign-in-visibility }

로컬 계정 레코드와 로그인 표시 설정은 신원 기반 지속성을 드러낼 수 있습니다. 이 검사들은 수집된 SAM 및 SOFTWARE 하이브를 검사하며, Active Directory 계정, 도메인 그룹 멤버십 또는 과거 로그온의 목록이 아닙니다.

### Administrators 그룹 멤버십 {#admin_group_membership}

**검사 ID:** `admin_group_membership` · **카탈로그 ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei가 검사하는 내용:** SAM에서 로컬 Administrators 별칭(alias)의 멤버 목록을 디코딩합니다. 수집된 SAM에서 숨겨진 계정을 가리키거나 어떤 계정/그룹도 가리키지 않는 로컬 머신 SID를 보고합니다.

**검사하는 증거:**

- SAM 하이브의 `SAM\Domains\Builtin\Aliases\00000220\C` 및 `SAM\Domains\Account` 아래의 머신 SID/사용자/그룹 레코드.
- 숨겨진 이름과의 상관 분석을 위한 `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`.

**조사 방법:** 이는 특정 이상 징후를 대상으로 한 검사이며, 모든 관리자의 목록이 아닙니다. 외부/도메인 SID는 누락된 로컬 사용자로 평가하지 않습니다. 악의적인 멤버십으로 결론 내리기 전에 삭제된 계정, 수집 시점 또는 불완전한 하이브를 조사해야 합니다.

### 숨겨진 로컬 계정 {#hidden_account}

**검사 ID:** `hidden_account` · **카탈로그 ATT&CK:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**Juntei가 검사하는 내용:** Winlogon SpecialAccounts UserList에서 숫자 0 값을 보고하여, 일반 로그인 화면에서 숨겨진 이름을 식별합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: 계정 이름 값.

**조사 방법:** 서비스 또는 유지 관리 계정은 의도적으로 숨겨질 수 있으며, 이 값은 지정된 계정이 존재하거나 관리자임을 입증하지 않습니다. SAM 계정 레코드 및 로그온 활동과 결합하여 확인하십시오.

### RID 하이재킹 {#rid_hijacking}

**검사 ID:** `rid_hijacking` · **카탈로그 ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei가 검사하는 내용:** SAM 계정 하위 키의 16진수 RID를 해당 F 값의 오프셋 0x30에 저장된 RID와 비교하여 불일치를 보고합니다.

**검사하는 증거:**

- SAM 하이브의 `SAM\Domains\Account\Users\<hexadecimal RID>\F`.

**조사 방법:** 불일치는 심각한 계정 레코드 이상 징후이지만, 손상/수집 분석이 필요할 수도 있습니다. 원본 바이트, 트랜잭션 로그 상태 및 계정/로그온 기록을 확인하십시오. 어떤 행위자가 레코드를 수정했는지는 증명하지 않습니다.

### RID Suborner(숨겨진 관리자 계정) {#rid_suborner}

**검사 ID:** `rid_suborner` · **카탈로그 ATT&CK:** [T1098](https://attack.mitre.org/techniques/T1098/)

**Juntei가 검사하는 내용:** 하위 키 RID가 500이 아니지만 F 값의 RID가 500인 SAM 계정을 보고합니다. 이는 관리자 신원을 대체하는 보다 구체적인 패턴입니다.

**검사하는 증거:**

- SAM 하이브의 `SAM\Domains\Account\Users\<hexadecimal RID>\F`.

**조사 방법:** 이는 일반 RID 불일치 검사와 중복됩니다. 카탈로그 제목과 달리 숨겨진 로그인 설정을 별도로 요구하지 않으므로, 계정 표시 여부, 권한 및 실제 계정 사용은 별도로 확인하십시오.

## 인증 구성 요소 및 자격 증명 보호 { #authentication-components-and-credential-protection }

LSA(Local Security Authority)는 LSASS 및 관련 구성 요소를 통해 보안 기능을 처리합니다. 인증 패키지, 암호 필터 및 네트워크 공급자는 정상적인 보안 소프트웨어일 수도 있고, 지속적인 자격 증명 접근 경로일 수도 있습니다. 두 개의 검사는 실행 파일 자동 시작이 아니라 보호 설정의 목록을 작성합니다. 호스트 역할, Windows 버전 및 유효 정책을 바탕으로 해석하십시오.

### AMSI 공급자 DLL {#amsi_providers}

**검사 ID:** `amsi_providers` · **카탈로그 ATT&CK:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**Juntei가 검사하는 내용:** 시스템 AMSI 공급자 CLSID를 열거하고, 가능한 경우 네이티브/32비트 InprocServer32 라이브러리를 확인합니다. 확인되지 않은 등록은 클래스 ID를 유지합니다.

**검사하는 증거:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` 및 확인 가능한 `Classes\CLSID` / `Classes\Wow6432Node\CLSID`의 InprocServer32 등록.

**조사 방법:** 바이러스 백신 및 보안 제품은 정상적으로 AMSI 공급자를 등록합니다. 공급자/라이브러리의 출처와 실제 로드를 검토하십시오. 등록 정보는 AMSI가 우회되었거나 공급자가 악성이라는 증거가 아닙니다.

### 인증 패키지 {#authentication_packages}

**검사 ID:** `authentication_packages` · **카탈로그 ATT&CK:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**Juntei가 검사하는 내용:** 활성 제어 집합에서 LSA `Authentication Packages`의 비어 있지 않은 개별 항목을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**조사 방법:** 원본 Windows 빌드 및 설치된 인증 소프트웨어와 비교하십시오. 기본값이 아닌 패키지도 정상일 수 있습니다. 가로채기로 결론 내리기 전에 DLL을 확인하고 로드/자격 증명 활동을 확증하십시오.

### DSRM 관리자 로그온 동작 {#dsrm_backdoor}

**검사 ID:** `dsrm_backdoor` · **카탈로그 ATT&CK:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**Juntei가 검사하는 내용:** 구현된 ControlSet001, ControlSet002 또는 CurrentControlSet 위치에서 숫자 `DsrmAdminLogonBehavior=2`를 보고합니다. 이 설정은 DSRM 계정을 일반적인 복구 상황 이외에서 사용할 수 있도록 허용할 수 있습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior`. 존재하는 경우 ControlSet002 및 CurrentControlSet도 포함합니다.

**조사 방법:** 주로 도메인 컨트롤러와 관련이 있습니다. SYSTEM Select를 사용하여 어떤 제어 집합이 활성 상태인지 확인하십시오. 과거 제어 집합에서 일치했다고 해서 반드시 유효한 것은 아닙니다. 이 값은 공격자가 DSRM 암호를 알고 있었거나 로그온했다는 증거가 아닙니다.

### Credential Guard 구성 {#lsa_cfg_flags}

**검사 ID:** `lsa_cfg_flags` · **카탈로그 ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei가 검사하는 내용:** 구성된 LSA `LsaCfgFlags` 값을 읽으며, 0뿐 아니라 일반적인 활성화 값도 포함합니다. 이는 실행 파일 자동 시작이 아니라 Credential Guard 구성의 목록을 작성하는 것입니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**조사 방법:** 이 값은 Windows 버전, 펌웨어/보안 전제 조건 및 유효 정책과 함께 해석하십시오. 저장된 0 값이나 값의 부재만으로는 공격자에 의한 변경, 유효한 보호 상태 또는 자격 증명 탈취가 입증되지 않습니다.

### LSA 암호 필터 {#lsa_password_filter}

**검사 ID:** `lsa_password_filter` · **카탈로그 ATT&CK:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**Juntei가 검사하는 내용:** LSA `Notification Packages`를 읽습니다. 경로 없는 라이브러리 이름은 수집된 Windows System32 디렉터리를 기준으로 확인하며, 필요한 경우 `.dll`을 추가합니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**조사 방법:** 암호 정책 제품이 흔히 필터를 설치합니다. 라이브러리와 로드 증거를 확인하고, 구성된 필터와 암호 변경 또는 자격 증명 가로채기가 발생했다는 증거를 구분하십시오.

### LSASS PPL 보호 상태 {#lsa_run_as_ppl}

**검사 ID:** `lsa_run_as_ppl` · **카탈로그 ATT&CK:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**Juntei가 검사하는 내용:** 0이 아닌 값을 포함하여 구성된 LSA `RunAsPPL` 값을 읽습니다. 이는 보호 설정의 목록이며, LSASS가 보호 없이 실행되었는지를 테스트하는 것이 아닙니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**조사 방법:** 실제 LSA 보호는 Windows 버전, 정책, 펌웨어 잠금 및 재시작 상태에 따라 달라집니다. 값이 없어 행이 출력되지 않은 것은 측정된 보호 결과가 아닙니다. 유효한 보호 상태는 독립적인 시스템/런타임 증거로 평가하십시오.

### 네트워크 공급자 DLL {#network_provider_dll}

**검사 ID:** `network_provider_dll` · **카탈로그 ATT&CK:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**Juntei가 검사하는 내용:** 활성 SYSTEM 제어 집합의 서비스 아래에서 `NetworkProvider\ProviderPath`를 읽습니다.

**검사하는 증거:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`.

**조사 방법:** 네트워크 로그온 공급자는 정상일 수 있습니다. 공급자 순서/적용 여부, 설치된 네트워크/보안 소프트웨어 및 실제 라이브러리 로드를 확인하십시오. ProviderPath 값만으로는 자격 증명 가로채기가 증명되지 않습니다.

### 보안 패키지 {#security_packages}

**검사 ID:** `security_packages` · **카탈로그 ATT&CK:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**Juntei가 검사하는 내용:** 구성된 보안 지원 공급자(SSP)를 다루는 LSA `Security Packages`의 비어 있지 않은 개별 항목을 읽습니다.

**검사하는 증거:**

| 레지스트리 키 | 검사하는 값 |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**조사 방법:** 예상되는 Windows/보안 제품 기준선과 비교하고 각 라이브러리를 확인하십시오. 구성은 공급자가 로드되었거나, 자격 증명을 수집했거나, 인시던트 기간 중에 추가되었다는 증거가 아닙니다.

## 후보에서 결론까지 { #work-from-a-candidate-to-a-conclusion }

1. `SourceFile`로 식별되는 원래 원본을 여십시오. 전체 값, XML 동작, 스크립트 또는 복구된
   데이터베이스 레코드와, 그것이 속한 정확한 사용자/호스트를 확인하십시오.
2. 메커니즘이 작동할 수 있었는지 판단하십시오. 활성화 플래그, 런타임 또는 Windows 버전,
   서비스/작업 상태, 필요한 트리거, 사용자, 아키텍처 및 활성 제어 집합을 확인하십시오.
3. 선택된 프로그램, DLL, 스크립트 또는 어셈블리를 확인하십시오. 수집된 바이트의 해시를 계산하고 인수와
   내용을 검토하십시오. 익숙한 Windows 실행 프로그램이나 게시자 문자열이라고 해서 정상 여부가 결정되지는 않습니다.
4. 독립적인 증거를 사용하여 실제로 무슨 일이 일어났는지 확인하십시오. 예를 들어 프로세스 생성,
   Sysmon 이미지 로드, 작업 스케줄러 런타임 기록, 서비스 활동, Prefetch 및 관련 파일이 있습니다.
   구성 변경 이벤트는 설정이 변경되었음을 증명할 수 있지만, 그 페이로드가 실행되었음을 증명하지는 않습니다.
5. 원래 원본의 타임스탬프와 기록된 상관 분석 출처를 사용하여 타임라인을 구성하십시오.
   키 쓰기, 설치 변경, 파일/컨테이너 쓰기 및 실행은 각각 별개의 사실로 유지하십시오.
6. 결과뿐 아니라 한계도 기록하십시오. 누락된 하이브, 누락된 대상 파일, 알 수 없는 작업 주체,
   검증되지 않은 서명자 메타데이터, 과거에 카빙된 레코드 또는 확증할 수 없는 트리거 등이 있습니다.

명령어 구문, 프로필, 모든 출력 필드 및 커버리지 상태에 대해서는
[Persistence Hunter 명령어 가이드](../commands/persistence-hunter.md)를 참조하십시오.
[아티팩트 가이드](../artifacts/index.md)는 기반이 되는 원본을 설명하며,
[감사의 말](acknowledgements.md)에서는 업스트림 지속성 도구를 소개합니다.
