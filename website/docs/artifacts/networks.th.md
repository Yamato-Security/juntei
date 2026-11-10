# เครือข่าย { #networks }

## สิ่งเหล่านี้คืออะไร { #what-they-are }

Windows จดจำเครือข่ายที่ระบบเคยเชื่อมต่อและวิธีการตั้งค่าเครือข่ายของระบบ

- **`SOFTWARE`:**
  - **โปรไฟล์เครือข่าย** (`NetworkList\Profiles`): หนึ่งโปรไฟล์ต่อหนึ่งเครือข่ายที่ระบบเคยเชื่อมต่อ
    พร้อมชื่อ (SSID ของเครือข่าย Wi-Fi) หมวดหมู่ (public, private, domain) ชนิดการเชื่อมต่อ
    (แบบมีสาย ไร้สาย VPN บรอดแบนด์มือถือ) และเวลาที่ระบบเชื่อมต่อครั้งแรกและครั้งล่าสุด
  - **Signature** (`NetworkList\Signatures`): สิ่งที่ใช้ระบุแต่ละเครือข่าย เช่น
    MAC address ของเกตเวย์และ DNS suffix
  - **การ์ดเครือข่าย** (`NetworkCards`): อะแดปเตอร์
- **`SYSTEM`** (ต่อ control set):
  - **อินเทอร์เฟซ TCP/IP:** แอดเดรส เซิร์ฟเวอร์ DHCP และ lease เซิร์ฟเวอร์ DNS โดเมน
  - **แชร์** ที่ระบบเปิดให้บริการ (`LanmanServer\Shares`)
  - **Port proxy** (`PortProxy`): อุโมงค์ที่ `netsh interface portproxy` ตั้งค่าไว้ ซึ่ง
    ผู้โจมตีใช้เพื่อ pivot และเพื่อเข้าถึง RDP ผ่านพอร์ตอื่น
  - **เส้นทางแบบถาวร (persistent route)**
  - **เขตเวลา** ซึ่งใช้แปลงเวลาท้องถิ่นของโปรไฟล์ให้เป็น UTC

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ระบบเชื่อมต่อกับเครือข่ายที่มีชื่อและเกตเวย์นี้
ครั้งแรกและครั้งล่าสุด ณ เวลาเหล่านี้ ระบบมีแอดเดรสนี้จากเซิร์ฟเวอร์ DHCP นี้
และเปิดแชร์และอุโมงค์เหล่านี้ไว้ ณ เวลาที่บันทึกไฮฟ์ ชื่อเครือข่ายเพียงอย่างเดียวไม่สามารถ
ระบุสถานที่ได้ เพราะเครือข่ายจำนวนมากมีชื่อว่า "Network" หรือใช้ SSID เดียวกัน

## ตำแหน่ง { #location }

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

ให้เก็บรวบรวมไฮฟ์ทั้งสองพร้อมบันทึกธุรกรรม (transaction log) ของไฮฟ์ ต้องใช้ไฮฟ์ SYSTEM เพื่อแปลงเวลาของโปรไฟล์
เป็น UTC

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **เวลาของโปรไฟล์เป็นเวลาท้องถิ่น** `DateCreated` และ `DateLastConnected` ถูกเขียนเป็นเวลาท้องถิ่นของ
  ระบบ
  - `FirstConnectedLocal` และ `LastConnectedLocal` เก็บค่าตามที่พบ
  - `FirstConnectedUTC` และ `LastConnectedUTC` แปลงค่าโดยใช้เขตเวลาจากไฮฟ์ SYSTEM ของระบบ
    (`TimeZone`) และกฎเวลาออมแสง (daylight saving) ของเขตเวลานั้น
  - เวลาที่อยู่ก่อนการเปลี่ยนเขตเวลา หรืออยู่ในปีที่ใช้กฎเวลาออมแสงต่างออกไป
    (วันที่ของสหรัฐฯ ก่อนปี 2007) อาจคลาดเคลื่อนตามผลต่างนั้น
  - หากไม่มีไฮฟ์ SYSTEM ที่อ่านได้และ current control set ที่ถูกต้อง จะแสดงเฉพาะเวลาท้องถิ่น ช่วงเวลาที่ขาดหายจาก DST และชั่วโมงที่ซ้ำกันอย่างกำกวมไม่มี timestamp UTC ที่พิสูจน์ได้เพียงค่าเดียว: UTC จะถูกปล่อยว่างและ `Problems` จะอธิบายข้อจำกัดนั้น offset หรือวันที่ที่ไม่ถูกต้องก็จะไม่ถูกแปลงเช่นกัน
- **มีเพียงครั้งแรกและครั้งล่าสุด** แต่ละโปรไฟล์มีการเชื่อมต่อครั้งแรกและครั้งล่าสุดอย่างละหนึ่งค่า ไม่ใช่ทุก
  การเชื่อมต่อ โดยปกติเวลาเขียนล่าสุดของคีย์โปรไฟล์จะตรงกับการเชื่อมต่อครั้งล่าสุด
- **ชื่อซ้ำกันได้** Windows ตั้งชื่อเครือข่ายแบบมีสายที่ไม่ได้รับการจัดการว่า "Network", "Network 2" และต่อไปเรื่อย ๆ
  SSID ของ Wi-Fi ไม่ได้มีค่าไม่ซ้ำกัน ให้ใช้ `GatewayMac` และ `DnsSuffix` เพื่อแยกแยะเครือข่าย และ
  เชื่อมโยงค่าเหล่านี้กับบันทึก DHCP และทะเบียนทรัพย์สิน (asset inventory) MAC ของเกตเวย์ไม่จำเป็นต้องเป็น BSSID ของ Wi-Fi access point และเพียงลำพังไม่สามารถยืนยันสถานที่ทางกายภาพได้
- **แชร์ proxy และเส้นทาง** เป็นการกำหนดค่า: แชร์หรือ port proxy อาจถูกตั้งค่าไว้และ
  ไม่เคยถูกใช้งานเลย แต่ละค่าไม่มีเวลาของตนเอง เวลาเขียนล่าสุดของคีย์คือเวลาที่ค่าใดค่าหนึ่ง
  เปลี่ยนแปลง
- **อินเทอร์เฟซ:** คีย์อินเทอร์เฟซภายใต้ GUID ที่ไม่มีแอดเดรสคืออะแดปเตอร์ที่ยังไม่ได้ตั้งค่า
  เมื่อใช้ DHCP แอดเดรสจะเป็นของ lease ล่าสุด แอดเดรสแบบ static ที่หลงเหลือจากก่อนใช้ DHCP
  จะไม่แสดงว่ากำลังใช้งานอยู่
- **Control set:** โดยค่าเริ่มต้นจะอ่าน control set ที่ Windows ใช้ล่าสุด `--all-control-sets`
  จะอ่านทุก control set ซึ่งอาจแสดงแชร์หรือ proxy ที่ถูกลบออกจาก control set ปัจจุบันแล้ว

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

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

ข้อมูลสรุปจะแสดงเครือข่ายที่เชื่อมต่อล่าสุด (ใหม่สุดก่อน) และแชร์ port proxy และ
เส้นทางแบบถาวรทุกรายการ

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | โฮสต์ และไฮฟ์ |
| `ControlSet`, `IsCurrentControlSet` | สำหรับเรคคอร์ดของ SYSTEM: control set และเป็น control set ที่ Windows ใช้ล่าสุดหรือไม่ |
| `Source` | `network_profile`, `network_signature` (signature ที่ไม่มีโปรไฟล์ใดใช้), `network_card`, `interface`, `share`, `port_proxy`, `route` |
| `ProfileName` | ชื่อของโปรไฟล์ (SSID ของเครือข่าย Wi-Fi) หรือคำอธิบายของ signature |
| `Category`, `ConnectionType` | `public`, `private`, `domain`; `wired`, `wireless`, `vpn`, `mobile_broadband` (หรือเป็นตัวเลขในกรณีอื่น) |
| `FirstConnectedUTC`, `LastConnectedUTC` | `DateCreated` และ `DateLastConnected` ที่แปลงเป็น UTC ด้วย `TimeZone` |
| `FirstConnectedLocal`, `LastConnectedLocal` | ค่าเดียวกันตามที่พบ (เวลาท้องถิ่น ละเอียดถึงมิลลิวินาที) |
| `TimeZone` | เขตเวลาที่ใช้ในการแปลง |
| `GatewayMAC`, `DNS-Suffix` | MAC address ของเกตเวย์และ DNS suffix จาก signature ของโปรไฟล์ (คั่นด้วย `|` หากมีหลายค่า) |
| `InterfaceGUID` | GUID ของอินเทอร์เฟซ หรือ `ServiceName` ของการ์ดเครือข่าย |
| `IP-Address`, `DhcpServer`, `LeaseObtainedUTC`, `LeaseExpiresUTC` | แอดเดรสที่อินเทอร์เฟซใช้งานอยู่ เซิร์ฟเวอร์ DHCP และ lease |
| `ShareName`, `SharePath` | แชร์และโฟลเดอร์ที่แชร์นั้นเปิดให้บริการ |
| `ListenAddress`, `ListenPort`, `ConnectAddress`, `ConnectPort` | port proxy: สิ่งที่รอรับการเชื่อมต่อ (listen) และปลายทางที่ส่งต่อไป |
| `Route` | เส้นทางแบบถาวรตามที่จัดเก็บไว้: `destination,netmask,gateway,metric` |
| `Details` | ค่าอื่น ๆ ในรูปแบบ `Name=Value; …` (CSV) หรืออ็อบเจกต์ (JSON): คำอธิบายของโปรไฟล์ `Managed` ชื่อแรกและคีย์ signature; อะแดปเตอร์ของอินเทอร์เฟซ (จาก `NetworkCards`) มาสก์ เกตเวย์ เซิร์ฟเวอร์ DNS และโดเมน; ชนิด หมายเหตุ และฟิลด์อื่นของแชร์; ชนิดของ proxy; ส่วนประกอบของเส้นทาง |
| `KeyPath`, `ValueName`, `KeyLastWrittenUTC` | คีย์ ค่าสำหรับแชร์ proxy หรือเส้นทาง และเวลาเขียนล่าสุดของคีย์ |
| `DataRaw` | ไบต์ที่ถอดรหัสไม่ได้: เวลาที่ไม่ใช่ `SYSTEMTIME` ที่ถูกต้อง หรือแชร์ที่ไม่ใช่รายการสตริง |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่มีชนิดหรือขนาดไม่ถูกต้อง วันที่ที่เป็นไปไม่ได้ และชื่อที่ไม่อยู่ในรูปแบบที่ควรเป็น |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงลำดับโปรไฟล์ตาม `LastConnectedUTC` และตรวจสอบเครือข่ายในช่วงเวลาที่เกิดเหตุ: เครือข่าย
   Wi-Fi ที่ไม่รู้จัก ฮอตสปอตมือถือ VPN
2. ใช้ `GatewayMac` และ `DnsSuffix` เพื่อแยกแยะเครือข่ายที่มีชื่อเดียวกัน และเพื่อจับคู่
   เครือข่ายเดียวกันข้ามหลายโฮสต์
3. ตรวจสอบ port proxy และแชร์ทุกรายการ port proxy ไปยังพอร์ต 3389 หรือ 445 ของโฮสต์ภายใน หรือแชร์
   ของโฟลเดอร์พักข้อมูล (staging) (`C:\ProgramData`, `C:\Users\Public`) เป็นเบาะแส ให้เชื่อมโยงกับ event
   log (การสร้างโพรเซส `netsh`, เหตุการณ์แชร์ 5142/5144) และเวลาของคีย์
4. ใช้แอดเดรสและเวลา lease ของอินเทอร์เฟซเพื่อระบุตำแหน่งของโฮสต์บนเครือข่าย และจับคู่
   กับบันทึกของไฟร์วอลล์ DHCP และ proxy
5. ใช้ `--all-control-sets` และ `parse-registry read --recover-deleted` สำหรับแชร์
   proxy และโปรไฟล์ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: netsh interface portproxy](https://learn.microsoft.com/en-us/windows-server/networking/technologies/netsh/netsh-interface-portproxy)
- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [MITRE ATT&CK T1090.001: Proxy: Internal Proxy](https://attack.mitre.org/techniques/T1090/001/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
