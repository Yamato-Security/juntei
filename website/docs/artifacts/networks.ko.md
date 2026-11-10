# 네트워크 { #networks }

## 개요 { #what-they-are }

Windows는 시스템이 연결했던 네트워크와 네트워크가 어떻게 구성되었는지를 기억합니다.

- **`SOFTWARE`:**
  - **네트워크 프로필** (`NetworkList\Profiles`): 시스템이 연결한 네트워크마다 하나씩 있으며, 이름(Wi-Fi
    네트워크의 SSID), 범주(공용, 개인, 도메인), 연결 유형(유선, 무선, VPN, 모바일 광대역), 그리고 시스템이
    처음과 마지막으로 연결한 시간을 가집니다.
  - **서명** (`NetworkList\Signatures`): 게이트웨이의 MAC 주소와 DNS 접미사 등 각 네트워크를 식별하는 정보.
  - **네트워크 카드** (`NetworkCards`): 어댑터.
- **`SYSTEM`** (제어 집합별):
  - **TCP/IP 인터페이스:** 주소, DHCP 서버와 임대, DNS 서버, 도메인.
  - 시스템이 제공하는 **공유** (`LanmanServer\Shares`).
  - **포트 프록시** (`PortProxy`): `netsh interface portproxy`가 설정하는 터널로, 공격자가 피벗하거나 다른
    포트를 통해 RDP에 접근하는 데 사용합니다.
  - **영구 경로.**
  - 프로필의 로컬 시간을 UTC로 변환하는 **표준 시간대**.

<span class="evidence-strength">가장 타당한 결론:</span> 시스템은 이 이름과 게이트웨이를 가진 네트워크에
처음과 마지막으로 이 시간에 연결했습니다. 하이브가 저장된 시점에 이 DHCP 서버로부터 이 주소를 받았으며, 이
공유와 터널을 제공하고 있었습니다. 네트워크 이름만으로는 그것이 어디였는지 식별할 수 없습니다. 많은
네트워크가 "Network"라는 이름을 갖거나 같은 SSID를 공유하기 때문입니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\NetworkList\Profiles\{GUID}
      ProfileName, Description, Category, NameType, Managed, DateCreated, DateLastConnected
  Microsoft\Windows NT\CurrentVersion\NetworkList\Signatures\Managed|Unmanaged\<signature>
      ProfileGuid, DefaultGatewayMac, DnsSuffix, FirstNetwork
  Microsoft\Windows NT\CurrentVersion\NetworkCards\<n>          ServiceName, Description
C:\Windows\System32\config\SYSTEM (ControlSet00N\…)
  Services\Tcpip\Parameters\Interfaces\{GUID}                    IP, DHCP, DNS, lease times
  Services\LanmanServer\Shares                                   one value per share
  Services\PortProxy\v4tov4|v4tov6|v6tov4|v6tov6\tcp             listen = connect
  Services\Tcpip\Parameters\PersistentRoutes                     destination,mask,gateway,metric
  Control\TimeZoneInformation
```

두 하이브를 트랜잭션 로그와 함께 수집하십시오. 프로필 시간을 UTC로 변환하려면 SYSTEM 하이브가 필요합니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **프로필 시간은 로컬 시간입니다.** `DateCreated`와 `DateLastConnected`는 시스템의 로컬 시간으로
  기록됩니다.
  - `FirstConnectedLocal`과 `LastConnectedLocal`은 발견된 그대로 보존합니다.
  - `FirstConnectedUTC`와 `LastConnectedUTC`는 시스템 SYSTEM 하이브의 표준 시간대(`TimeZone`)와 일광 절약
    시간 규칙으로 변환합니다.
  - 표준 시간대가 변경되기 전의 시간이나, 다른 일광 절약 시간 규칙이 적용되던 해(2007년 이전의 미국 날짜)의
    시간은 그 차이만큼 어긋날 수 있습니다.
  - 읽을 수 있는 SYSTEM 하이브와 유효한 현재 제어 집합이 없으면 로컬 시간만 제공됩니다. DST 공백 구간과
    모호하게 반복되는 시간대에는 증명 가능한 단일 UTC 타임스탬프가 없으므로, UTC는 비워 두고 `Problems`에
    그 한계를 설명합니다. 잘못된 오프셋이나 날짜도 변환되지 않은 상태로 남습니다.
- **처음과 마지막만.** 각 프로필에는 모든 연결이 아니라 첫 연결과 마지막 연결 하나씩만 있습니다. 프로필 키의
  마지막 쓰기 시간은 대개 마지막 연결과 일치합니다.
- **이름은 반복됩니다.** Windows는 관리되지 않는 유선 네트워크에 "Network", "Network 2" 등의 이름을
  붙입니다. Wi-Fi SSID는 고유하지 않습니다. `GatewayMac`과 `DnsSuffix`로 네트워크를 구별하고, 이를 DHCP
  로그 및 자산 목록과 상관 분석하십시오. 게이트웨이 MAC이 반드시 Wi-Fi 액세스 포인트의 BSSID인 것은 아니며,
  그것만으로 물리적 위치를 확정할 수 없습니다.
- **공유, 프록시, 경로**는 구성 정보입니다. 공유나 포트 프록시는 설정만 되고 한 번도 사용되지 않을 수
  있습니다. 각 값에는 자체 시간이 없으며, 키의 마지막 쓰기 시간은 그중 하나라도 변경된 시점입니다.
- **인터페이스:** 주소가 없는 GUID 아래의 인터페이스 키는 설정되지 않은 어댑터입니다. DHCP를 사용하는 경우
  주소는 마지막 임대의 것입니다. DHCP 이전부터 남아 있는 정적 주소는 사용 중인 것으로 표시되지 않습니다.
- **제어 집합:** 기본적으로 Windows가 마지막으로 사용한 제어 집합을 읽습니다. `--all-control-sets`는 모든
  제어 집합을 읽으며, 현재 제어 집합에서 제거된 공유나 프록시를 보여 줄 수 있습니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry networks <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and SYSTEM hives
  -f, --file <FILE>      Parse one hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry networks -d evidence/ -o networks.csv
juntei parse-registry networks -d evidence/ -t jsonl \
  | jq 'select(.Source == "port_proxy" or .Source == "share")'
```

요약에는 마지막으로 연결된 네트워크(최신순)와 모든 공유, 포트 프록시, 영구 경로가 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `SourcePath` | 호스트와 하이브. |
| `ControlSet`, `IsCurrentControlSet` | SYSTEM 레코드의 경우: 제어 집합, 그리고 그것이 Windows가 마지막으로 사용한 제어 집합인지 여부. |
| `Source` | `network_profile`, `network_signature` (어떤 프로필에도 속하지 않는 서명), `network_card`, `interface`, `share`, `port_proxy`, `route`. |
| `ProfileName` | 프로필의 이름(Wi-Fi 네트워크의 SSID). 서명의 경우 설명. |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (그 외에는 숫자). |
| `FirstConnectedUTC`, `LastConnectedUTC` | `TimeZone`으로 UTC 변환한 `DateCreated`와 `DateLastConnected`. |
| `FirstConnectedLocal`, `LastConnectedLocal` | 같은 값을 발견된 그대로(로컬 시간, 밀리초 단위) 표시. |
| `TimeZone` | 변환에 사용된 표준 시간대. |
| `GatewayMAC`, `DNS-Suffix` | 프로필 서명의 게이트웨이 MAC 주소와 DNS 접미사(여러 개일 때는 `|`로 구분). |
| `InterfaceGUID` | 인터페이스의 GUID. 네트워크 카드의 경우 `ServiceName`. |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | 인터페이스의 사용 중인 주소, DHCP 서버 및 임대. |
| `ShareName`, `SharePath` | 공유와 그것이 제공하는 폴더. |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | 포트 프록시: 수신 대기하는 곳과 전달하는 곳. |
| `Route` | 저장된 그대로의 영구 경로: `destination,netmask,gateway,metric`. |
| `Details` | 기타 값. `Name=Value; …` (CSV) 또는 개체(JSON) 형식: 프로필의 설명, `Managed`, 첫 이름 및 서명 키; 인터페이스의 어댑터(`NetworkCards`에서), 마스크, 게이트웨이, DNS 서버 및 도메인; 공유의 유형, 설명 및 기타 필드; 프록시의 종류; 경로의 구성 요소. |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | 키, 공유·프록시·경로의 값, 그리고 키의 마지막 쓰기 시간. |
| `DataRaw` | 디코딩되지 않는 바이트: 유효한 `SYSTEMTIME`이 아닌 시간, 문자열 목록이 아닌 공유. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트. |
| `Problems` | 잘못된 유형이나 크기의 값, 불가능한 날짜, 형식에 맞지 않는 이름. |

## 조사 전략 { #investigation-strategy }

1. 프로필을 `LastConnectedUTC`로 정렬하고 인시던트 전후의 네트워크를 확인합니다. 알 수 없는 Wi-Fi 네트워크,
   모바일 핫스팟, VPN 등입니다.
2. `GatewayMac`과 `DnsSuffix`를 사용하여 같은 이름의 네트워크를 구별하고, 여러 호스트에 걸쳐 같은 네트워크를
   대조합니다.
3. 모든 포트 프록시와 공유를 확인합니다. 내부 호스트의 3389나 445로 향하는 포트 프록시, 또는 스테이징 폴더
   (`C:\ProgramData`, `C:\Users\Public`)의 공유는 단서가 됩니다. 이벤트 로그(`netsh` 프로세스 생성,
   5142/5144 공유 이벤트) 및 키 시간과 상관 분석하십시오.
4. 인터페이스의 주소와 임대 시간을 사용하여 호스트의 네트워크상 위치를 파악하고, 방화벽, DHCP, 프록시 로그와
   대조합니다.
5. 제거된 공유, 프록시, 프로필은 `--all-control-sets`와 `parse-registry read --recover-deleted`를 사용하여
   찾습니다.

## 추가 정보 { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
