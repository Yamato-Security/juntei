# 설치된 프로그램 { #installed-programs }

## Windows가 보관하는 정보 { #what-windows-keeps }

Windows와 설치 프로그램은 설치된 소프트웨어를 여러 레지스트리 목록에 기록합니다. 각 목록에는 프로그램,
제품, 배포판 또는 패키지마다 하나의 키가 있으며, Juntei는 키마다 하나의 레코드를 제공합니다.

| 원본 | 하이브와 키 | 담고 있는 내용 |
|---|---|---|
| `uninstall` | `SOFTWARE` 및 `NTUSER.DAT`의 `…\CurrentVersion\Uninstall\*`, 그리고 `SOFTWARE\Wow6432Node\…` | 설정과 *프로그램 및 기능*에 표시되는 정보: 이름, 버전, 게시자, 설치 날짜, 폴더, 제거 명령, 크기, 숨김 여부 |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT`의 `Software\Microsoft\Installer\Products\*` | Windows Installer 제품: 제품 이름과 버전, 패키지 파일 이름 및 설치에 사용된 원본 폴더 |
| `app_path` | `SOFTWARE` 및 `NTUSER.DAT`의 `…\CurrentVersion\App Paths\*` | Windows가 이름으로 시작하는 프로그램(`Win+R`, `start tool`): 프로그램의 경로와 폴더 |
| `wsl` | `NTUSER.DAT`의 `…\CurrentVersion\Lxss\{GUID}` | Linux용 Windows 하위 시스템(WSL) 배포판: 이름, 파일 또는 디스크의 위치, WSL 1 또는 2 |
| `package` | `SOFTWARE`의 `…\Appx\AppxAllUserStore` | 패키지(Store) 앱: 모든 사용자용으로 프로비저닝된 앱, Windows 기본 앱, 스테이징된 앱, 프로비저닝 해제된 앱, 그리고 각 사용자의 앱 |

<span class="evidence-strength">가장 타당한 결론:</span> 이 프로그램, 제품 또는 패키지가 모든 사용자용
또는 이 사용자용으로 시스템에 설치(또는 등록)되었습니다. 설치 날짜는 설치 프로그램이 기록한 값이며, 키의
마지막 쓰기 시간은 해당 엔트리가 마지막으로 변경된 시점입니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

`SOFTWARE`, 모든 `NTUSER.DAT` 및 각각의 트랜잭션 로그를 수집하십시오. `SYSTEM`에서 호스트 이름을 알 수
있고, `SOFTWARE`의 `ProfileList`에서 각 사용자의 SID를 알 수 있습니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **설치 날짜와 키 시간.** `InstallDate`는 설치 프로그램이 로컬 시간 기준으로 기록한 날짜(시각 없음)입니다.
  일부 설치 프로그램은 이 값을 기록하지 않고, 일부는 다른 형식으로 기록합니다(`InstallDateRaw`에 기록된
  그대로 보존됨). 업데이트는 날짜를 바꾸지 않고 엔트리를 다시 쓰거나, 프로그램이 거의 바뀌지 않았는데
  날짜만 바꿀 수 있습니다. `InstallDate`와 `KeyLastWrittenUTC`를 비교하십시오.
- **제거된 소프트웨어.** 깨끗하게 제거된 프로그램은 키가 사라집니다. 삭제된 `Uninstall` 키는 `parse-registry read
  --recover-deleted`를 사용하고, 남아 있는 흔적은 Amcache, Prefetch 및 `$MFT`를 사용하십시오.
- **사용자별 등록.** `NTUSER.DAT` 엔트리는 프로필에 등록된 소프트웨어를 식별합니다. 이 범위만으로는 누가
  설치 프로그램을 실행했는지, 관리자 권한이 사용되었는지는 확정되지 않습니다.
- **숨겨진 엔트리.** `SystemComponent`는 엔트리를 *프로그램 및 기능*에서 숨깁니다. `DisplayName`이 없는
  엔트리(업데이트, 구성 요소)도 표시되지 않습니다. 이들은 모두 보고됩니다.
- **제품 코드.** GUID 형태의 `Uninstall` 키는 `ProductCode`를 통해 압축(packed) 형식의 `msi_product`
  키와 매칭할 수 있습니다. GUID 형태의 이름만으로는 Windows Installer가 엔트리를 생성했다고 증명할 수
  없습니다.
- **App Paths.** 이는 프로그램을 찾는 데 사용되는 등록 정보이며 실행 기록이 아닙니다. `Path` 값에는 여러
  검색 디렉터리가 들어 있을 수 있으며, 반드시 설치 디렉터리인 것은 아닙니다.
- **게시자.** 이는 설치 프로그램이 제공한 레지스트리 메타데이터이며, 검증된 서명 정보가 아닙니다.
- **아키텍처.** `Wow6432Node`에는 64비트 Windows의 32비트 프로그램이 들어 있습니다(`Architecture` `x86`,
  `Wow64` `True`). 다른 엔트리는 시스템 자체의 아키텍처입니다(비어 있음). 패키지의 아키텍처는 패키지
  자체의 것입니다(`x64`, `neutral`, …).
- **WSL.** `Version`은 배포판의 `Flags`에서 얻은 WSL 버전입니다(`0x8`: WSL 2의 가상 머신에서 실행됨).
  배포판 자체의 `Version` 값(`Details`에 포함)은 파일 시스템의 버전이며, WSL 1 배포판에서도 2입니다.
  `BasePath`에는 `ext4.vhdx`(WSL 2) 또는 `rootfs`(WSL 1)가 있습니다. 내부를 살펴보려면 이를 수집하십시오.
- **패키지.** `<SID>` 목록은 각 사용자에게 등록된 패키지이며, 대부분은 Windows 자체의 패키지입니다.
  `Deprovisioned` 패키지는 새 사용자에 대해 제거된 것이고, `EndOfLife\<SID>` 패키지는 해당 사용자에 대해
  제거된 것입니다.
- **손상된 값.** 잘못된 형식의 숫자, 잘못된 값 유형 및 유효하지 않은 UTF-16은 `Problems`에 보고되고
  `DataRaw`에 보존됩니다. 값이 존재하지만 0바이트인 경우에는 빈 16진수 문자열로 보존됩니다. 선택적 숫자
  필드의 정상적인 빈 문자열은 값이 없는 것으로 취급됩니다. 패키지 버전은 각각 부호 없는 16비트 범위 내의
  10진수 구성 요소 네 개로 이루어져야 합니다. `-t`가 레코드를 stdout으로 스트리밍할 때 하이브 수준의
  경고는 stderr에 출력됩니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

요약에는 가장 최근에 설치된 프로그램(`InstallDate` 기준), 사용자가 자신을 위해 설치한 프로그램, 그리고
WSL 배포판이 나열됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath`, `HiveType` | 호스트, 하이브 파일, 그리고 `SOFTWARE` 또는 `NTUSER.DAT`입니다. |
| `User`, `UserSID` | 사용자 하이브의 사용자(프로필 폴더와 `ProfileList` SID), 또는 패키지가 등록된 사용자입니다. |
| `Source`, `Scope` | 목록(`uninstall`, `msi_product`, `app_path`, `wsl`, `package`)과 `machine` 또는 `user`입니다. |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; 제품의 `ProductName`과 `Version`; App Path의 프로그램 파일 이름; 배포판의 이름과 WSL 버전; 패키지의 이름과 버전입니다. |
| `InstallDate`, `InstallDateRaw` | `YYYYMMDD` 형식으로 기록된 경우 날짜로 표시한 `InstallDate`이며, 그렇지 않으면 기록된 그대로입니다. |
| `KeyLastWrittenUTC` | 엔트리의 키가 마지막으로 변경된 시점입니다. |
| `InstallLocation`, `InstallSource` | `InstallLocation`(App Path의 `Path`, 배포판의 `BasePath`, 패키지의 `Path`); `InstallSource`(제품이 마지막으로 사용한 원본 폴더)입니다. |
| `Program` | App Path의 프로그램입니다. |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString`입니다. |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | KiB 단위의 `EstimatedSize`, 숨김 여부, Windows Installer로 설치되었는지 여부입니다. |
| `ProductCode` | Windows Installer 제품 코드(`{GUID}`)입니다. |
| `Architecture`, `WOW64` | `Wow6432Node` 아래이면 `x86`, 또는 패키지의 아키텍처이며, 키가 `Wow6432Node` 아래에 있는지 여부입니다. |
| `Details` | 추가 정보: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User`(Inno Setup 설치 프로그램을 실행한 계정); 제품의 `PackageCode`, `PackageName`; 배포판의 `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default`; 패키지의 `List`, `ResourceId`, `PublisherId`입니다. |
| `KeyPath` | 엔트리의 키입니다. |
| `DataRaw` | 이름이 나타내는 형식과 다른 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 읽을 수 없거나 이름이 나타내는 형식과 다른 값입니다. |

## 조사 전략 { #investigation-strategy }

1. 사고 시점 전후로 `InstallDate`와 `KeyLastWrittenUTC`를 기준으로 정렬하고, 원격 액세스 도구, 터널링 및
   파일 전송 클라이언트, 압축 프로그램, 스크립트 런타임 및 보안 도구를 살펴보십시오.
2. `Scope`가 `user`인 레코드, 즉 관리자 권한 없이 설치된 소프트웨어를 검토하십시오.
3. `InstallLocation`, `InstallSource`, `UninstallCommand`에서 사용자가 쓰기 가능한 폴더(`Downloads`,
   `AppData`, `Temp`, `ProgramData`)를 확인하십시오.
4. WSL 배포판을 찾고, 해당 `BasePath`를 수집하십시오.
5. Amcache 및 Prefetch(설치 프로그램과 프로그램의 실행), `$MFT`(설치 폴더의 생성), 그리고 Application
   이벤트 로그(MsiInstaller 이벤트)와 상관 분석하십시오.

## 추가 정보 { #further-details }

- [Microsoft: Uninstall 레지스트리 키](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: 앱 등록(App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: 패키지 ID 개요](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
