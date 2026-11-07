# 지원하는 증거 유형 { #supported-evidence }

| 증거 출처 | Juntei 명령어 | 가장 유용한 용도 | 중요한 제한 사항 |
|---|---|---|---|
| Amcache | `parse-registry amcache` | 파일 인벤토리, 경로, SHA-1, PE 메타데이터, 장치 | 존재는 실행이 아니며, 게시자는 서명 검증이 아님 |
| NTFS `$MFT` | `parse-ntfs mft` | 파일, 삭제된 레코드, 경로, 타임스탬프, ADS | 재사용된 레코드와 타임스톰핑으로 인해 오해할 수 있음 |
| NTFS `$LogFile` | `parse-ntfs logfile` | redo/undo 메타데이터, 이전 이름과 타임스탬프, USN 저널에 대한 쓰기 | 순환 구조로 보존 기간이 짧음, 메타데이터 시간은 트랜잭션 시간이 아님, 재생하지 않음 |
| Prefetch | `parse-file prefetch` | 프로그램 실행, 실행 횟수, 최근 실행 시간, 로드된 파일 | 제한된 실행 시간 기록만 보존됨 |
| USN 저널 `$J` | `parse-ntfs usn-journal` | 생성, 삭제, 이름 변경, 쓰기, 과거 경로 | 순환 저널이며, 파일 내용이 아니라 변경 사항을 기록함 |
| 디렉터리 인덱스 `$I30` | `parse-ntfs i30` | 디렉터리 내 파일, 인덱스 슬랙에 남아 있는 삭제되었거나 이름이 변경되었을 수 있는 파일의 이름, 시간, 크기 | 슬랙은 부분적이며 엔트리가 언제 제거되었는지 알려 주지 않음 |
| NTFS 부트 섹터 `$Boot` | `parse-ntfs boot` | LNK, Jump Lists, Prefetch가 기록하는 볼륨 일련 번호, 지오메트리와 `$MFT` 위치, 호스트 간 비교를 위한 부트 코드 해시 | 일련 번호는 변경되거나 중복될 수 있음, 부트 코드가 다른 것은 단서이지 부트킷의 증명이 아님 |
| NTFS 보안 설명자 `$Secure:$SDS` | `parse-ntfs secure-sds` | `$MFT`의 `SID`에 따른 파일의 소유자와 ACL, NULL DACL과 허용 범위가 넓은 ACL | 소유자는 변경될 수 있음, ACL은 누가 액세스할 수 있는지를 나타낼 뿐 누가 액세스했는지가 아님 |
| VHDX 디스크 이미지 | `mount`, `auto-parse --vhdx` | NTFS 볼륨의 파일을 Juntei나 다른 도구가 그 자리에서 읽기 전용으로 읽음 | FUSE 필요(Linux 전용), 대체 데이터 스트림은 표시되지 않음 |
| SRUM | `parse-ese srum` | 앱/사용자별 네트워크 및 리소스 사용량 | 보통 집계된 값이며, 패킷이나 목적지 수준의 증명이 아님 |
| 사용자 액세스 로깅 (UAL) | `parse-ese ual` | 어떤 클라이언트 주소와 계정이 Windows Server의 역할(파일 공유, Active Directory, DHCP, IIS)을 어느 날에 사용했는지 | 요청은 하나씩 기록되지 않고 날짜별로 집계됨, 주소가 NAT나 프록시일 수 있음, 더티 상태의 `Current.mdb`에는 최신 변경 사항이 빠져 있음 |
| LNK 바로 가기 | `parse-file lnk` | 열린 대상, 인수, 볼륨, 공유, 원본 머신 | 대상이 실행되지 않아도 바로 가기가 생성될 수 있음 |
| Jump Lists | `parse-file jump-lists` | 애플리케이션이 연 항목, 고정, 사용 내역 및 포함된 LNK 데이터 | 애플리케이션별로 동작과 보존 방식이 다름 |
| ShimCache | `parse-registry shimcache` | 과거 실행 파일 경로와 호환성 검사 | 종료될 때까지 캐시가 플러시되지 않을 수 있음, 엔트리만으로는 실행이 아님 |
| Defender 격리 데이터 | `parse-file defender-quarantine` | 탐지, 원래 경로, 해시, 다운로드 출처, 복구된 바이트 | 탐지 이름은 제품의 판정임, 추출은 위험함 |
| 레지스트리 하이브 | `parse-registry read` | 구성, 지속성, 삭제/재생된 값 | 키 시간은 값의 시간이 아님, 라이브 하이브에는 트랜잭션 로그가 필요함 |
| 서비스 | `parse-registry services` | 서비스/드라이버 지속성과 실패 시 동작 | 구성은 서비스가 시작되었음을 증명하지 않음 |
| RecentDocs | `parse-registry recent-docs` | 사용자별 최근에 연 파일과 폴더 | 각 목록에서 가장 최근 항목만 근거 있는 열기 시간을 얻음 |
| Microsoft Office | `parse-registry office` | 각 Office 애플리케이션이 연 문서와 폴더 및 그 시간, 사용자가 매크로를 활성화한 문서, 매크로 설정과 신뢰할 수 있는 위치 | 매크로 활성화는 매크로가 실행되었다는 증명이 아님, 신뢰 레코드의 경로는 Office가 기록한 그대로 저장됨 |
| 열기 및 다른 이름으로 저장 대화 상자 | `parse-registry open-save-mru` | 공용 대화 상자를 통해 열거나 저장한 파일(확장자별), 그리고 이를 사용한 프로그램과 폴더 | 각 목록의 가장 최근 엔트리만 날짜가 있음, 열기와 저장을 구분하지 않음 |
| TaskCache | `parse-registry taskcache` | 예약된 작업의 동작, 메타데이터, 숨겨진 작업 | 런타임 증명을 위해 작업 XML 및 이벤트 로그와 상관 분석 필요 |
| 부팅 구성 (BCD) | `parse-registry bcd` | 부팅 항목, 복구 비활성화, 실패 무시, 테스트 서명, 무결성 검사 비활성화, 안전 모드와 각각이 설정된 시점 | 저장소가 수집되지 않는 경우가 많음(EFI 시스템 파티션, `\Boot`), 설정은 상속됨 |
| 레지스트리 자동 실행 | `parse-registry persistence` | Run 키, Winlogon, IFEO, AppInit/AppCert DLL, LSA 패키지, COM 하이재킹 및 기타 자동 실행 위치 | 구성이지 실행의 증명이 아님, 대부분의 엔트리는 Windows 기본값임 |
| 애플리케이션 호환성 | `parse-registry app-compat` | 호환성 레이어, 프로그램 호환성 관리자가 실행을 확인한 프로그램, CIT 프로그램 및 모듈 사용, 사용자 지정 shim 데이터베이스 | 프로그램 호환성 관리자 엔트리에는 실행 시간이 없음, CIT 데이터는 대부분 Windows 7 및 8의 것임 |
| FeatureUsage, RecentApps, 고정 항목 | `parse-registry feature-usage` | 사용자가 각 애플리케이션으로 전환, 실행 또는 클릭한 횟수, 최근 앱과 그 파일, 작업 표시줄 고정 항목과 시작 메뉴 프로그램 | 횟수에는 시간이 없음, 시작이 아니라 창 전환을 집계함 |
| ShellBags | `parse-registry shell-bags` | 사용자가 탐색기와 대화 상자에서 연 폴더(이동식, 네트워크, 휴대용 장치, ZIP 및 삭제된 폴더 포함)의 전체 경로 | 각 키에서 가장 최근 항목만 근거 있는 열기 시간을 얻음, 항목 시간은 방문 시간이 아니라 폴더의 시간임 |
| BAM 및 DAM | `parse-registry bam` | 각 사용자가 각 프로그램을 마지막으로 실행한 시간(이동식 및 네트워크 장치에서의 실행 포함) | 프로그램 및 사용자당 하나의 시간, 최근 활동만 기록, 드라이브 문자가 아니라 장치 경로 |
| USB 장치 | `parse-registry usb` | USB 저장 장치 및 기타 장치, 일련 번호, 설치/연결/제거 시간, 드라이브 문자, 볼륨 이름과 일련 번호, 장치를 마운트한 사용자 | 마지막 연결과 제거만 보존됨, 드라이브 문자는 재사용됨 |
| MuiCache | `parse-registry muicache` | 사용자의 셸 프로필과 연관된 캐시된 프로그램 경로, 설명 및 회사 | 프로그램별 시간이 없음, 프로필이 반드시 프로세스 계정인 것은 아니며 메타데이터는 위조될 수 있음 |
| UserAssist | `parse-registry user-assist` | 사용자가 탐색기에서 시작한 프로그램과 바로 가기, 실행 횟수, 포커스 시간 및 마지막 실행 | 탐색기에서의 시작만 집계됨, 새 프로필에는 기본 항목이 미리 채워짐 |
| 네트워크 | `parse-registry networks` | 연결한 네트워크와 최초/마지막 연결 시간, 게이트웨이 MAC, 인터페이스와 DHCP 임대, 공유, 포트 프록시, 경로 | 프로필 시간은 로컬 시간이며 현재 표준 시간대로 변환됨, "Network 2" 같은 이름은 반복됨 |
| 보안 설정 | `parse-registry security-settings` | Defender(및 그 제외 항목), UAC, WDigest/LSA 보호, RDP, 방화벽, SMBv1, PowerShell 로깅 및 AMSI, 이벤트 로그 및 증거 관련 설정(기본값을 약화시키는 경우 표시) | 약화된 설정은 침해의 증명이 아님, 존재하지 않는 값(Windows 기본값)은 보고되지 않음 |
| 애플리케이션 기록 | `parse-registry app-history` | 사용자가 실행한 Sysinternals 도구, 압축 및 폴더 기록(WinRAR, 7-Zip, WinZip), OneDrive 계정, Adobe 최근 파일, TeamViewer 및 CCleaner 값 | 대부분의 목록 엔트리에는 시간이 없음, 각 목록의 가장 최근 엔트리만 날짜가 있음 |
| MRU 목록 | `parse-registry mru` | 실행 대화 상자 명령, 탐색기에 입력한 경로, 탐색기 검색, Internet Explorer에 입력한 주소(시간 포함), 그림판 및 워드패드 파일, XP 검색어 | 입력한 주소를 제외하면 각 목록의 가장 최근 엔트리만 날짜가 있음, 입력했다고 실행된 것은 아님 |
| 원격 액세스 (사용자 측) | `parse-registry remote-access` | 사용자가 원격 데스크톱, PuTTY, WinSCP로 접속한 컴퓨터, 수락한 SSH 호스트 키, 매핑된 드라이브와 공유 | 저장되거나 입력된 대상이 성공한 연결은 아님, 목록 키에는 모든 항목에 대해 하나의 시간만 있음 |
| 설치된 프로그램 | `parse-registry installed-programs` | 설치된 프로그램(머신 전체 및 사용자별), Windows Installer 제품, App Paths, WSL 배포판 및 패키지 앱 | 설치 날짜는 설치 프로그램이 기록한 값임, 깨끗하게 제거된 프로그램은 엔트리를 남기지 않음 |
| SAM | `parse-registry sam` | 로컬 계정, 플래그, RID, 로그온/암호 메타데이터 | 로컬 계정만 해당, 마지막 로그온 값은 완전한 로그온 기록이 아님 |
| LSA 비밀 정보 | `parse-registry lsa-secrets` | LSA가 보유한 비밀 정보(자동 로그온, 서비스 계정, 컴퓨터 계정)와 설정 시점, 사용 중인 캐시된 도메인 로그온 | 이름과 시간만 제공하며 아무것도 복호화하지 않음, `DefaultPassword`는 자동 로그온 없이도 존재함 |
| 감사 정책 | `parse-registry auditpol` | 시스템이 기록하도록 설정된 보안 이벤트, 따라서 이벤트가 없어도 아무것도 의미하지 않는 경우 | 하이브가 마지막으로 기록된 시점의 정책임, 변경 사항은 이벤트 4719로 기록됨 |
| 시스템 정보 | `parse-registry system-info` | 컴퓨터 및 도메인 이름, Windows 버전과 설치 날짜, 표준 시간대, 마지막 종료, 마지막 사용자, 프로필, SID | 수집 시점의 구성임, `InstallDate`는 기능 업그레이드 시 바뀌며 Windows 11은 자신을 Windows 10으로 표시함 |

## 수집 유형 { #collection-types }

Juntei는 `-f`로 단일 아티팩트를 받거나 `-d`로 수집본을 재귀적으로 검색합니다. 디렉터리 모드는 각 호스트에
`C/Windows/System32/config/SYSTEM`이나 `C/Users/alice/NTUSER.DAT` 같은 경로가 들어 있는 일반적인 트리아지
구조를 위해 설계되었습니다.

복사된 레지스트리나 ESE 데이터베이스를 분석하기 전에 [증거 수집](../getting-started/collection.md)을
참조하십시오. Windows가 보았던 상태를 재구성하려면 관련 트랜잭션 로그가 필수적일 수 있습니다.
