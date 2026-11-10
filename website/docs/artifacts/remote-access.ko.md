# 원격 액세스 (사용자 측) { #remote-access-user-side }

## 개요 { #what-it-is }

사용자의 `NTUSER.DAT`에는 사용자가 해당 컴퓨터에서 수행한 원격 연결의 흔적이 남습니다. 이는 측면 이동의
출발지 측 흔적입니다.

- **원격 데스크톱(`mstsc`):**
  - 클라이언트에 입력한 컴퓨터(가장 최근 순)
  - 사용자가 로그인한 컴퓨터마다 하나의 키(사용한 계정 이름과 사용자가 수락한 서버 인증서의 지문 포함)
  - 사용자가 로컬 장치 사용을 허용한 컴퓨터
- **PuTTY:**
  - 저장된 세션: 호스트, 포트, 계정, 프로토콜, 키 파일, 프록시 및 포트 포워딩
  - 마지막으로 시작한 세션
  - 사용자가 수락한 모든 SSH 서버 키
- **WinSCP:** PuTTY와 같은 형식의 저장된 세션 및 수락된 SSH 서버 키
- **매핑된 드라이브:** 로그인 시 복원되도록 매핑된 각 드라이브와, *네트워크 드라이브 연결*에 입력한 경로
- **공유:** Explorer가 마운트한 공유(`MountPoints2`)

이 흔적은 원격 컴퓨터 자체의 로그가 사라진 후에도 계정이 다음에 어디로 갔고 어떤 자격 증명을 사용했는지를
보여 줍니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 사용자의 프로필이 이 대상에 연결했거나 이
대상으로의 연결을 저장했습니다. 저장된 세션이나 입력된 이름만으로는 연결이 성공했음을 증명하지 못합니다.
수락된 호스트 키나 `UsernameHint`가 있는 `Servers` 키는 더 강력한 증거로, 클라이언트가 서버에 도달했음을
나타냅니다.

## 위치 { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Terminal Server Client\Default            MRU0 … MRU9
Software\Microsoft\Terminal Server Client\Servers\<host>     UsernameHint, CertHash
Software\Microsoft\Terminal Server Client\LocalDevices       <host> = devices
Software\SimonTatham\PuTTY\Sessions\<session>
Software\SimonTatham\PuTTY\Jumplist                          Recent sessions
Software\SimonTatham\PuTTY\SshHostKeys                       <type>@<port>:<host> = key
Software\Martin Prikryl\WinSCP 2\Sessions\<session>
Software\Martin Prikryl\WinSCP 2\SshHostKeys
Network\<letter>                                             RemotePath, UserName, ProviderName
Software\Microsoft\Windows\CurrentVersion\Explorer\Map Network Drive MRU
Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2\##server#share
```

모든 프로필의 `NTUSER.DAT`를 트랜잭션 로그와 함께 수집하십시오. 각 사용자의 SID를 제공하는 시스템의
`SOFTWARE`도 수집하십시오.

## 분석 시 주의 사항 { #analyst-cautions }

- **키 시간은 목록의 시간입니다.** `Default`, `SshHostKeys` 및 `Map Network Drive MRU`는 목록을 담고
  있으며, 키의 마지막 쓰기 시간은 목록이 마지막으로 변경된 시간입니다. 그 시간은 첫 번째 항목
  (`MruPosition` 0)에 해당하며, 나머지 항목에는 상한값일 뿐입니다. `Servers\<host>`, 세션,
  `Network\<letter>` 또는 공유 키는 자체 시간을 가집니다.
- **저장되었다고 사용된 것은 아닙니다.** PuTTY나 WinSCP 세션은 한 번도 연결하지 않고 저장될 수 있습니다.
  PuTTY는 저장된 세션이 시작될 때 `Recent sessions`를 다시 쓰고, 서버에 처음 도달했을 때 호스트 키를
  추가합니다.
- **비밀번호:**
  - WinSCP는 세션, 터널 및 프록시 비밀번호를 저장할 수 있습니다.
  - PuTTY는 프록시의 비밀번호만 평문으로 저장합니다.
  - `PasswordStored`와 `Details`의 `StoredPasswords`는 어떤 값이 설정되어 있는지를 나타냅니다. 이 값들은
    절대 디코딩되거나 출력되지 않습니다.
- **WinSCP는 기본값을 생략합니다.** `FSProtocol`이 없는 세션은 SFTP이며, `PortNumber`가 없는 세션은
  프로토콜의 기본 포트를 사용합니다. 이 경우 `Protocol`과 `Port`는 추측하지 않고 비워 둡니다.
- **호스트 키 지문**은 `ssh-keygen -l`, `ssh` 및 PuTTY 0.75 이상이 표시하는 OpenSSH의 `SHA256:…`
  형식입니다. Juntei는 PuTTY와 WinSCP가 보관하는 키로부터 이를 계산합니다. 이름이나 IP 주소 뒤에 있는
  서버를 식별하려면 서버 자체의 키, `known_hosts` 파일 및 로그와 비교하십시오. SSH-1 키(`rsa`)에는
  지문이 없습니다.
- **매핑된 드라이브:** `RemoteUsername`은 드라이브가 다른 계정으로 매핑된 경우에만 설정됩니다. 사용자 자신의
  계정을 사용한 경우 Windows는 `REG_DWORD` 0을 기록합니다.
- **공유:** `MountPoints2`에는 볼륨과 드라이브 문자에 대한 키도 있습니다. 여기서는 네트워크 공유
  (`##server#share`)만 보고되며, `\\server@SSL@443\DavWWWRoot`는 WebDAV 공유입니다.
- **삭제:** 목록을 지우거나 포터블 클라이언트를 사용하면 여기에 흔적이 남지 않습니다. Jump Lists, Prefetch,
  `.rdp` 파일, RDP 비트맵 캐시 및 대상의 로그와 비교하십시오.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry remote-access <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry remote-access -d evidence/ -o remote.csv
juntei parse-registry remote-access -f NTUSER.DAT -t jsonl \
  | jq 'select(.Client == "rdp" or .HostKeyFingerprint != null)'
```

요약은 클라이언트별 연결 수를 집계합니다. 키가 마지막으로 기록된 연결을 최신 순으로 나열하고, 비밀번호를
보관하는 세션도 나열합니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | 호스트, 프로필 폴더 이름, 시스템의 `ProfileList`에서 얻은 사용자 SID(시스템의 `SOFTWARE` 하이브가 있는 경우), 그리고 하이브입니다. |
| `Client` | `rdp`, `putty`, `winscp`, `mapped_drive` 또는 `network_share`입니다. |
| `Kind` | 레코드의 출처입니다. `rdp_mru`, `rdp_server`, `rdp_local_devices`, `putty_session`, `putty_host_key`, `winscp_session`, `winscp_host_key`, `network_drive`, `map_network_drive_mru`, `mount_point`. |
| `Target`, `Port` | 컴퓨터(호스트 이름 또는 IP 주소)와 포트, 또는 공유(`\\server\share`)입니다. 포트는 Windows나 클라이언트가 기록한 경우에만 제공됩니다. |
| `RemoteUsername` | 대상에서 사용한 계정입니다. `UsernameHint`, 세션의 `UserName`(또는 PuTTY `HostName`의 `user@`), 매핑된 드라이브의 `UserName`. |
| `Protocol` | PuTTY의 `Protocol`(`ssh`, `telnet`, `raw`, …), WinSCP의 `FSProtocol`(`sftp`, `scp`, `ftp`, `webdav`, `s3`), 매핑된 드라이브의 공급자(`Microsoft Windows Network`)입니다. |
| `MRU-Position` | 목록 내 위치(0이 가장 최근)입니다. `MRU<n>`, `MRUList`, PuTTY의 `Recent sessions`. |
| `SessionName` | 저장된 세션의 이름으로, `%20` 및 기타 이스케이프가 디코딩됩니다. |
| `HostKeyType`, `HostKeyFingerprint`, `HostKey` | 수락된 SSH 호스트 키의 유형(`rsa2`, `ssh-ed25519`, `ecdsa-sha2-nistp256`, …), OpenSSH 지문, 그리고 저장된 그대로의 수치입니다. |
| `DriveLetter` | 매핑된 드라이브의 문자입니다. |
| `PasswordStored` | 세션이 비밀번호를 보관하는지 여부이며, 절대 디코딩되지 않습니다. |
| `Details` | 기타 설정으로, `Name=Value; …`(CSV) 또는 객체(JSON) 형식입니다. RDP 인증서의 `CertHash`, `LocalDevices`, 세션의 키 파일, 프록시, SSH 터널, 포트 포워딩, 원격 명령 및 디렉터리, `StoredPasswords`, 공유의 `Label`, 드라이브의 `ConnectionType`. |
| `KeyPath`, `ValueName` | 키와, 하나의 값으로 이루어진 레코드의 경우 그 값입니다. |
| `KeyLastWrittenUTC` | 키의 마지막 쓰기 시간입니다(목록에 관해서는 주의 사항 참조). |
| `DataRaw` | 값이 문자열이 아니거나 디코딩되지 않는 경우 값의 바이트(16진수)입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 잘못된 유형의 값(바이트 포함), 읽을 수 없는 호스트 키, 존재하지 않는 `MRUList` 문자, 그리고 나열되지 않은 값입니다. |

## 조사 전략 { #investigation-strategy }

1. 각 사용자의 대상을 클라이언트별로 나열하고, 그 사용자가 접근해야 하는 시스템과 대조하십시오. 관리 공유
   (`C$`, `ADMIN$`)와 IP 주소를 찾으십시오.
2. 사용된 계정(`RemoteUsername`)에 주목하십시오. 도메인 관리자와 다른 머신의 로컬 계정이 그 예입니다.
3. `HostKeyFingerprint`를 서버의 키와, `CertHash`를 RDP 서버의 인증서와 대조하십시오.
4. 대상의 로그온 이벤트(4624 유형 10 및 3, 1149), Jump Lists와 LNK 파일(`mstsc.exe`, `putty.exe`, 공유에서
   연 파일), Prefetch 및 RDP 비트맵 캐시와 상관 분석하십시오.
5. 삭제된 항목은 `parse-registry read --recover-deleted`로, 지워진 목록은 프로필의 이전 `NTUSER.DAT`
   사본(볼륨 섀도 복사본, 백업)으로 확인하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 원격 데스크톱 연결의 컴퓨터 상자에서 항목 제거](https://learn.microsoft.com/en-us/troubleshoot/windows-server/remote/remove-entries-from-remote-desktop-connection-computer)
- [PuTTY FAQ: PuTTY는 데이터를 어디에 저장합니까?](https://the.earth.li/~sgtatham/putty/0.83/htmldoc/AppendixA.html#faq-settings)
- [WinSCP: 구성 저장소](https://winscp.net/eng/docs/config)
- [libyal winreg-kb: Terminal Server Client](https://github.com/libyal/winreg-kb/blob/main/docs/sources/application-keys/Terminal-server-client.md)
- [libyal winreg-kb: Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
