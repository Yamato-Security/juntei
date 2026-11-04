# USB 장치 { #usb-devices }

## USB 기록이란 { #what-the-usb-history-is }

Windows는 설치하는 모든 장치를 기록합니다. USB 저장 장치, 기타 USB 장치 및 휴대용 장치(휴대전화, 카메라,
미디어 플레이어)의 경우 `SYSTEM` 하이브는 장치마다 키를 하나씩 두고 그 이름, 일련번호, 그리고 Windows가
장치를 설치한 시간, 마지막으로 연결을 감지한 시간, 마지막으로 제거를 감지한 시간을 보관합니다.
`MountedDevices`는 장치의 볼륨에 부여된 볼륨 GUID와 드라이브 문자를 기록합니다. `SOFTWARE` 하이브는
Explorer가 휴대용 장치와 볼륨에 부여한 이름과, ReadyBoost가 기록한 볼륨 일련번호를 보관합니다. 각 사용자의
`NTUSER.DAT`는 해당 사용자가 마운트한 볼륨을 기록합니다.

`parse-registry usb`는 이들을 결합하여 장치 및 볼륨당 하나의 레코드를 만듭니다. 어떤 장치였는지, 언제
설치되고 연결되었는지, 어떤 드라이브 문자와 볼륨을 가졌는지, 그리고 어떤 사용자가 그 볼륨을 마운트했는지가
포함됩니다.

<span class="evidence-strength">가장 타당한 결론:</span> 이 공급업체, 제품 및 일련번호를 가진 장치가 이
시스템에 연결되었으며, 처음과 마지막 연결은 이 시간이었습니다. 그 볼륨에는 이 드라이브 문자가 있었고, 이
사용자들이 볼륨을 마운트했습니다. 레지스트리 레코드는 어떤 파일이 복사되었는지 보여 주지 않습니다.

## 위치 { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| 장치 속성 | 필드 | 의미 (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: 드라이버가 마지막으로 설치된 시간 |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: 장치가 처음 설치된 시간 |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: 마지막으로 연결된 시간(Windows 8 이상) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: 마지막으로 제거된 시간(Windows 8 이상) |

Windows 8 이상은 각 시간을 `0064` … `0067` 키의 기본값으로 보관합니다. Windows 7은 이를
`00000064\00000000\Data`에 보관합니다. Juntei는 두 가지를 모두 읽습니다.

많은 DFIR 참고 자료는 `0064`를 최초 설치로, `0065`를 설치로 표기합니다. Juntei는 위의 Microsoft
`devpkey.h` 정의를 따르며, 각 시간이 기록하는 내용에 맞춰 열 이름을 붙입니다.

`-f` 또는 `-d`에는 `SYSTEM` 하이브를 지정하십시오. 시스템의 `SOFTWARE` 하이브와 사용자들의 `NTUSER.DAT`
하이브는 수집 데이터에 보관된 형태(`Windows\System32\config\SOFTWARE`, `Users\<user>\NTUSER.DAT`)로 그
옆에서 찾거나, 하이브를 함께 내보낸 경우 같은 디렉터리와 한 단계 아래에서 찾습니다. 모든 하이브를 트랜잭션
로그와 함께 수집하십시오.

## 레코드 결합 방식 { #how-the-records-are-joined }

- **장치:** 장치 키당 하나의 레코드이며, `usbstor`, `usb` 및 `wpdbusenum`(`DeviceClass`)이 있습니다. USB 저장
  장치는 같은 컨테이너 ID(없으면 같은 일련번호)를 가진 USB 장치로부터 공급업체 및 제품 ID를 가져옵니다.
- **볼륨:** USB 저장 장치의 볼륨은 데이터가 해당 장치를 지정하는 경로인 `MountedDevices`의
  `\??\Volume{GUID}` 값입니다. 볼륨이 두 개인 장치에는 레코드가 두 개 있습니다. 드라이브 문자는 같은 데이터를
  가진 `\DosDevices\X:` 값입니다.
- **이름과 일련번호:** 볼륨 이름은 `Windows Portable Devices`와 같은 장치를 지정하는 휴대용 장치에서
  가져옵니다. 볼륨 레이블과 일련번호는 해당 장치를 지정하는 `EMDMgmt` 키에서 가져옵니다.
- **사용자:** `MountPoints2`에 볼륨의 GUID가 있는 사용자이며, 해당 키의 마지막 쓰기 시간이 함께 표시됩니다.
- **장치 키가 없는 볼륨**(`volume`): 어떤 장치 키도 지정하지 않는 `MountedDevices` 볼륨(내부 디스크의 볼륨,
  또는 `Enum` 키가 사라진 제거된 장치의 볼륨), 어떤 볼륨 GUID 값과도 데이터를 공유하지 않는 드라이브 문자,
  그리고 사용자의 `MountPoints2`에만 있는 볼륨입니다.
- **`SOFTWARE`에만 이름이 있는 장치**(`portable_device`, `readyboost`).

`SourceKeys`는 레코드를 결합하는 데 사용된 모든 키와 값을 하이브와 함께 나열합니다.

## 분석 시 주의 사항 { #analyst-cautions }

- **드라이브 문자는 재사용됩니다.** `MountedDevices`는 각 문자를 마지막으로 부여받은 장치를 보관합니다.
  문자는 데이터가 여전히 그 문자의 데이터와 같은 장치에만 표시됩니다.
- **일련번호는 만들어진 값일 수 있습니다.** 인스턴스의 두 번째 문자가 `&`이면, 장치가 일련번호를 보고하지
  않아 Windows가 만든 값이며, 이 경우 `Serial`은 비어 있습니다. 이러한 인스턴스는 다른 컴퓨터에서 다를 수
  있습니다.
- **USB 저장 장치 인스턴스는 논리 장치 번호로 끝납니다**(`&0`). `Serial`은 이를 생략하고, `Instance`는 이를
  유지합니다.
- **시간은 Windows 버전에 따라 다릅니다.** Windows 7에는 설치 시간만 있습니다. `LastArrivalUTC`와
  `LastRemovalUTC`는 마지막 연결만을 나타내며, 모든 연결을 나타내지는 않습니다. 이벤트 로그
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`)와
  `setupapi.dev.log`에서 더 많은 정보를 얻을 수 있습니다.
- **제어 집합:** 기본적으로 Windows가 마지막으로 사용한 제어 집합을 읽습니다. `--all-control-sets`는 모든
  제어 집합을 읽으며, 이전 제어 집합에는 현재 제어 집합에 더 이상 없는 장치가 여전히 남아 있을 수 있습니다.
- **키의 마지막 쓰기 시간은 연결 시간이 아닙니다.** Windows는 여러 가지 이유로 장치 키를 다시 씁니다.
- **제거된 장치는 흔적을 남깁니다.** 공급업체와 제품은 있지만 장치 키가 없는 볼륨 레코드는, `Enum` 키가
  (제거 작업이나 정리 도구로) 삭제되었지만 `MountedDevices`에 그 볼륨이 아직 남아 있는 장치입니다.
- **ReadyBoost 일련번호**는 볼륨의 일련번호(`VolumeSerials`, `dir`이 표시하는 형식)이며, LNK 파일과 Jump
  Lists도 이를 기록합니다.

## Juntei로 파싱하기 { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

요약에는 마지막으로 연결된 USB 저장 장치가 드라이브 문자와 함께 최신 순으로 나열되며, 장치 키가 없는 USB
저장 장치에 속한 볼륨 수도 표시됩니다.

## 출력 필드 { #output-fields }

| 필드 | 해석 |
|---|---|
| `Host` | `SYSTEM`에서 얻은 컴퓨터 이름입니다. |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum`, 또는 장치 키가 없는 경우 `volume`, `portable_device`, `readyboost`입니다. |
| `ControlSet` | 장치 키의 제어 집합입니다. |
| `Vendor`, `Product`, `Revision` | USB 저장 장치: 장치 이름(`Disk&Ven_…&Prod_…&Rev_…`)에서 얻으며, 볼륨이나 `SOFTWARE` 키가 지정하는 장치에도 적용됩니다. |
| `Vid`, `PID` | USB 공급업체 및 제품 ID입니다. |
| `Serial`, `Instance` | 일련번호와, 기록된 그대로의 인스턴스 키 이름입니다. |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | 기록된 그대로의 장치 키 값입니다(`DeviceDesc`는 흔히 `@<inf>,%…%;<text>` 형식). 휴대용 장치의 `FriendlyName`은 보통 볼륨의 레이블이나 드라이브입니다. |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | 장치 속성 시간입니다(위 참조). |
| `DriveLetters` | `MountedDevices` 데이터가 해당 볼륨의 데이터와 같은 드라이브 문자입니다. |
| `VolumeGUID`, `VolumeData` | 볼륨의 GUID와, `MountedDevices` 데이터가 나타내는 볼륨의 위치입니다. 장치 경로, MBR 디스크 서명과 파티션 오프셋, 또는 GPT 파티션입니다. |
| `VolumeNames`, `VolumeSerials` | Explorer가 장치나 볼륨에 부여한 이름, ReadyBoost 레이블, 볼륨 일련번호(`1234-ABCD`)입니다. |
| `User` | `MountPoints2`에 해당 볼륨이 있는 사용자와 그 키의 마지막 쓰기 시간입니다(JSON: `User`, `MountPoints2LastWritten`, `SourcePath`). |
| `FoundIn`, `SourceKeys` | 발견된 위치(`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`)와, 결합에 사용된 모든 키와 값입니다. |
| `KeyPath`, `KeyLastWrittenUTC` | 장치의 인스턴스 키와 그 마지막 쓰기 시간입니다. |
| `SourcePath`, `SoftwarePath` | `SYSTEM` 하이브와, 정보를 제공한 경우 `SOFTWARE` 하이브입니다. |
| `TransactionLogStatus`, `LogsApplied` | 레코드 결합에 사용된 하이브의 레지스트리 복구 컨텍스트입니다. |
| `Problems` | 읽을 수 없었던 키와 FILETIME이 아닌 장치 속성(바이트 포함)입니다. |

## 조사 전략 { #investigation-strategy }

1. USB 저장 장치를 `LastArrivalUTC`(없으면 `DriverLastInstalledUTC`)로 정렬하고, 컴퓨터 자체의 리더와 허브는
   제외하십시오.
2. 각 장치의 공급업체, 제품, 일련번호, 볼륨 GUID, 드라이브 문자 및 볼륨 일련번호를 기록하고, LNK 파일, Jump
   Lists, ShellBags 및 `$MFT`(해당 드라이브 문자나 볼륨상의 경로)에서 이를 찾으십시오.
3. `User`에서 누가 볼륨을 마운트했는지, 그리고 언제(`MountPoints2` 키의 마지막 쓰기 시간)인지 확인하십시오.
4. USB 저장 장치를 지정하는 `volume` 레코드, 즉 키가 제거된 장치를 찾으십시오.
5. 마지막 연결뿐 아니라 모든 연결에 대해 이벤트 로그와 상관 분석하십시오.
6. 삭제된 장치 키를 찾으려면 `parse-registry read --recover-deleted`를 사용하십시오.

## 추가 정보 { #further-details }

- [Microsoft: 장치 속성 (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: 마운트 관리자 요청 지원](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (Wine 헤더 기준)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): 장치 속성 번호
- [libyal winreg-kb: USB storage](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) 및
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

사용자 결합에는 각 하이브의 원본 식별 정보가 사용되므로, 같은 프로필 이름을 가진 내보낸 하이브도 서로
구분됩니다. `SourceKeys`는 기여한 사용자 하이브 경로를 보존합니다. 함께 사용되는 사용자 하이브를 찾을 때는
재귀적 하이브 탐색과 마찬가지로 심볼릭 링크를 건너뜁니다. VID/PID 상관 분석에는 일치하는 사용 가능한 컨테이너
ID가 필요하며, 그것이 없으면 일련번호 매칭으로 모호하지 않은 ID가 도출되어야 합니다. 상충하는 후보는 문제와
함께 알 수 없는 상태로 남습니다.
