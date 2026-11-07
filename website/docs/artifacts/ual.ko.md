# 사용자 액세스 로깅 (UAL) { #user-access-logging-ual }

## UAL이란 { #what-ual-is }

사용자 액세스 로깅(UAL)은 Windows Server 2012부터 기본적으로 설치되고 활성화되어 있는 Windows Server
기능으로, 클라이언트가 서버의 역할에 보낸 요청을 집계하여 ESE 데이터베이스에 저장합니다. 역할, 클라이언트
주소, 계정의 조합마다 해당 연도에 클라이언트가 처음과 마지막으로 확인된 시각, 총 요청 수, 연중 각 날짜의
요청 수를 보관합니다. 역할에는 File Server(SMB), Active Directory Domain Services, Active Directory
Certificate Services, DHCP Server, Print and Document Services, Remote Access, Web Server(IIS),
Windows Server Update Services가 있습니다.

UAL은 측면 이동 조사에 유용합니다. 클라이언트 행은 어떤 주소와 계정이 어느 날에 서버를 사용했는지를
보여 주며, 이벤트 로그가 순환되어 사라진 뒤에도 오랫동안 남아 있는 경우가 많습니다.

<span class="evidence-strength">가장 타당한 결론:</span> 기록된 주소의 클라이언트가 기록된 계정으로 인증하여,
요청 수가 기록된 각 날짜에 해당 역할을 사용했습니다.

## 위치와 수집 { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb`에는 가장 최근의 활동이 들어 있습니다. `SystemIdentity.mdb`는 서버를 설명하고, 역할의 이름을
정의하며, 각 연도의 데이터를 담은 `{GUID}.mdb` 파일을 나열합니다(`CHAINED_DATABASES`). UAL 서비스는
데이터베이스를 계속 열어 두므로, 포렌식 수집 도구를 사용하거나 볼륨 스냅샷에서 `Sum` 디렉터리 전체를
수집하십시오.

!!! warning "더티 데이터베이스"
    실행 중인 서버에서 수집한 `Current.mdb`는 더티 상태인 경우가 많습니다. Juntei는 이를 수집된 그대로
    읽고 그 상태를 보고하지만, `Svc*.log`를 재생하지는 않습니다. 트랜잭션 로그에만 존재하는 활동은
    누락됩니다.

## 분석 시 주의 사항 { #analyst-cautions }

- UAL은 요청을 날짜별로 집계할 뿐, 각 요청을 개별적으로 기록하지 않습니다. 클라이언트 행에 있는 시각은
  `InsertDate`와 `LastAccess`뿐입니다.
- `DailyAccesses`의 날짜는 UTC 기준 날짜입니다.
- `Address`는 서버가 본 주소입니다. 서버와 실제 클라이언트 사이에 NAT 게이트웨이, 프록시 또는 VPN이 있을
  수 있습니다. `::1`과 `127.0.0.1`은 서버 자신입니다.
- `Sum` 디렉터리의 모든 데이터베이스를 파싱하십시오. 같은 클라이언트가 `Current.mdb`와 해당 연도의
  `{GUID}.mdb` 양쪽에 행을 가질 수 있습니다.
- 행이 없다고 해서 아무것도 연결하지 않았다는 증명이 되지는 않습니다. UAL은 비활성화할 수 있으며
  (`Disable-Ual` 또는 User Access Logging Service 중지), 더티 데이터베이스에는 최신 변경 사항이 빠져
  있습니다.
- `SystemIdentity.mdb`가 없으면 `RoleName`에는 Windows Server가 역할에 부여한 이름이 사용됩니다.
- `LastAccess`가 0이거나 유효하지 않으면 연도를 판단하는 데 사용하지 않습니다. Juntei는 유효한 `InsertDate`, 그다음 연간 데이터베이스의 메타데이터를 사용하며, 유효한 연도가 없으면 `DayOfYear`를 유지합니다.
- 요약 합계는 모든 원본 행을 합산합니다. `Current.mdb`, 연간 데이터베이스 및 스냅샷이 겹칠 수 있으므로 중복을 제거한 연결 또는 로그온 횟수로 해석하면 안 됩니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-ese ual <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Current.mdb, {GUID}.mdb and SystemIdentity.mdb
  -f, --file <FILE>      Parse one UAL database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese ual -d Sum -o ual-csv
juntei parse-ese ual -d Sum -t jsonl \
  | jq 'select(.Table == "CLIENTS") | {Address, AuthenticatedUserName, RoleName, LastAccess}'
```

## 출력 테이블 { #output-tables }

CSV는 테이블마다 하나의 파일을 생성하며, 테이블이 비어 있으면 헤더만 있는 파일이 됩니다. JSON 및 JSONL 행에는
`SourceFile`(데이터베이스)와 `Table`(데이터베이스 내 테이블 이름)이 포함됩니다.

| CSV 파일 | 테이블 | 내용 |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | 각 역할의 클라이언트: 주소, 계정, 해당 연도의 첫 액세스와 마지막 액세스, 총 요청 수와 일별 요청 수. |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | 각 역할이 처음과 마지막으로 사용된 시각. |
| `UAL-DNS.csv` | `DNS` | 주소와 호스트 이름의 쌍, 그리고 각 쌍이 마지막으로 확인된 시각. |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | Hyper-V 가상 머신: 식별자, 일련 번호, 생성 시각과 마지막 활성 시각. |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | 서버 자체: 이름, 도메인, Windows 버전, 제품 유형, 하드웨어, 표준 시간대, 마지막 부팅. 새 행은 이름 변경, 도메인 가입 또는 도메인 컨트롤러로의 승격을 나타낼 수 있습니다. |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | 각 연도의 `{GUID}.mdb` 파일. |
| `UAL-Roles.csv` | `ROLE_IDS` | 역할 GUID와 그 이름. |
| `UAL-<table>.csv` | 기타 테이블 | Juntei가 알지 못하는 테이블의 모든 열. |

## 필드 { #fields }

| 필드 | 해석 |
|---|---|
| `SourceFile` | 행을 읽어 온 데이터베이스. |
| `RoleGUID`, `RoleName` | 역할과, 데이터베이스 옆에 있는 `SystemIdentity.mdb`에서 가져온 그 이름(해당 파일이 없으면 기본 제공 이름). |
| `TenantID` | 역할이 보고하는 경우, 클라이언트가 속한 테넌트의 GUID. |
| `TotalAccesses` | 데이터베이스 연도 동안 클라이언트가 보낸 요청 수. `DailyAccesses`의 합계입니다. |
| `InsertDate` | 행이 생성된 시각. 즉, 이 데이터베이스에서 해당 연도에 클라이언트가 보낸 첫 요청입니다(UTC). |
| `LastAccess` | 클라이언트의 가장 최근 요청(UTC). |
| `Address` | 클라이언트의 IPv4 또는 IPv6 주소. |
| `AuthenticatedUserName` | 클라이언트가 인증에 사용한 계정(`DOMAIN\user`. 컴퓨터 계정은 `$`로 끝남). |
| `ClientName` | 역할이 보고하는 경우, 클라이언트의 이름. |
| `DailyAccesses` | 요청이 있었던 날짜와 그 요청 수. `Day1` … `Day366` 열에서 가져와 `LastAccess`의 연도로 날짜를 지정합니다. CSV에서는 `2023-01-12: 5, 2023-01-13: 2`, JSON에서는 `Date`와 `Accesses`를 가진 객체입니다. 연도를 알 수 없는 날짜에는 `Date` 대신 `DayOfYear`가 있습니다. |
| `FirstSeen`, `LastSeen` | 역할의 첫 사용과 마지막 사용(`ROLE_ACCESS`), DNS 쌍이 마지막으로 확인된 시각(`DNS`). 모두 UTC입니다. |
| `CreationTime`, `LastSeenActive` | 가상 머신 또는 `SYSTEM_IDENTITY` 행이 생성된 시각과, 가상 머신이 마지막으로 활성 상태였던 시각(UTC). |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server`. |
| `OS-CurrentTimeZone` | UTC를 기준으로 한 서버의 오프셋(분 단위, `-420`은 UTC-07:00). |
| `OS-LastBootUpTime` | 서버의 마지막 부팅 시각(UTC). 읽을 수 없으면 저장된 WMI 텍스트 그대로입니다. |
| 기타 열 | 테이블의 그 밖의 모든 열을 Juntei의 PascalCase 필드 레이블로 출력합니다. 예: `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber`, `SystemSerialNumber`. |

## 자동 분석 { #automatic-parsing }

`auto-parse`는 테이블을 `LateralMovement`에 `UAL-Clients.csv`, `UAL-RoleAccess.csv` 등으로 기록하며(`-t json`
또는 `-t jsonl`을 사용하면 하나의 `UAL.json` 또는 `UAL.jsonl` 파일), `LateralMovement/UAL-logs.txt`도 함께
생성합니다. 결합된 `-j` 타임라인에는 모든 행이 `Comp`와 `Artifact` 식별자 `ual`과 함께 포함됩니다. `-Z`는
`InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`, `CreationTime`, `LastSeenActive`, `OS-LastBootUpTime`을
변환합니다.

## 추가 정보 { #further-details }

- [Microsoft — User Access Logging](https://learn.microsoft.com/en-us/windows-server/administration/user-access-logging/get-started-with-user-access-logging)
- [Plaso — User Access Logging parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/user_access_logging.py)
- [Velociraptor — Windows.Forensics.UserAccessLogs](https://docs.velociraptor.app/artifact_references/pages/windows.forensics.useraccesslogs/)
- [CrowdStrike — How to Leverage User Access Logging for Forensic Investigations](https://www.crowdstrike.com/en-us/blog/user-access-logging-ual-overview/)
