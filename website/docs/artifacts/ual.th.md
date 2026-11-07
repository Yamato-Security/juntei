# การบันทึกการเข้าถึงของผู้ใช้ (UAL) { #user-access-logging-ual }

## UAL คืออะไร { #what-ual-is }

การบันทึกการเข้าถึงของผู้ใช้ (User Access Logging หรือ UAL) เป็นฟีเจอร์ของ Windows Server ที่ติดตั้งและเปิดใช้งานโดยค่าเริ่มต้นตั้งแต่
Windows Server 2012 ซึ่งรวบรวมคำขอที่ไคลเอนต์ส่งไปยังบทบาท (role) ต่าง ๆ ของเซิร์ฟเวอร์ไว้ในฐานข้อมูล
ESE สำหรับแต่ละบทบาท แอดเดรสของไคลเอนต์ และบัญชี UAL จะเก็บเวลาที่พบไคลเอนต์ครั้งแรกและครั้งล่าสุด
ในปีนั้น จำนวนคำขอทั้งหมด และจำนวนคำขอของแต่ละวันในปี บทบาทที่บันทึกได้แก่
File Server (SMB), Active Directory Domain Services, Active Directory Certificate Services, DHCP
Server, Print and Document Services, Remote Access, Web Server (IIS) และ Windows Server Update
Services

UAL มีคุณค่าสำหรับการสืบสวนการเคลื่อนย้ายภายในเครือข่าย (lateral movement): แถวของไคลเอนต์แสดงว่าแอดเดรสและ
บัญชีใดใช้งานเซิร์ฟเวอร์ และในวันใดบ้าง ซึ่งมักยังคงอยู่แม้ event log จะถูกเขียนทับวนรอบไปนานแล้ว

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ไคลเอนต์ที่แอดเดรสที่บันทึกไว้
ซึ่งยืนยันตัวตนเป็นบัญชีที่บันทึกไว้ ได้ใช้งานบทบาทนั้นในทุกวันที่มีจำนวนคำขอ

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\Windows\System32\LogFiles\Sum\Current.mdb
C:\Windows\System32\LogFiles\Sum\{GUID}.mdb          (one per year)
C:\Windows\System32\LogFiles\Sum\SystemIdentity.mdb
C:\Windows\System32\LogFiles\Sum\Svc*.log, Svc.chk   (transaction logs)
```

`Current.mdb` เก็บกิจกรรมล่าสุด `SystemIdentity.mdb` อธิบายเซิร์ฟเวอร์ ระบุชื่อบทบาท
และแสดงรายการไฟล์ `{GUID}.mdb` ที่เก็บข้อมูลของแต่ละปี (`CHAINED_DATABASES`) บริการ UAL
เปิดฐานข้อมูลค้างไว้ตลอด ดังนั้นให้เก็บรวบรวมไดเรกทอรี `Sum` ทั้งหมดด้วยเครื่องมือเก็บรวบรวมทางนิติวิทยาศาสตร์หรือจาก
สแนปช็อตของวอลุ่ม

!!! warning "ฐานข้อมูลที่อยู่ในสถานะ dirty"
    `Current.mdb` มักอยู่ในสถานะ dirty เมื่อเก็บรวบรวมจากเซิร์ฟเวอร์ที่กำลังทำงาน Juntei อ่านไฟล์นี้ตามที่เก็บรวบรวมมา
    และรายงานสถานะนั้น แต่ไม่เล่นซ้ำ `Svc*.log` กิจกรรมที่มีอยู่เฉพาะในบันทึกธุรกรรมจึง
    ขาดหายไป

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- UAL นับคำขอเป็นรายวัน ไม่ได้บันทึกคำขอทีละรายการ เวลาที่มีอยู่ในแถวไคลเอนต์มีเพียง
  `InsertDate` และ `LastAccess`
- วันใน `DailyAccesses` เป็นวันตาม UTC
- `Address` คือแอดเดรสที่เซิร์ฟเวอร์เห็น เกตเวย์ NAT พร็อกซี หรือ VPN อาจคั่นอยู่ระหว่างเซิร์ฟเวอร์กับ
  ไคลเอนต์จริง `::1` และ `127.0.0.1` คือตัวเซิร์ฟเวอร์เอง
- ให้วิเคราะห์ทุกฐานข้อมูลในไดเรกทอรี `Sum` เพราะไคลเอนต์เดียวกันอาจมีแถวทั้งใน `Current.mdb` และใน
  `{GUID}.mdb` ของปีนั้น
- การไม่มีแถวไม่ได้พิสูจน์ว่าไม่มีใครเชื่อมต่อ เพราะ UAL สามารถถูกปิดได้ (ด้วย `Disable-Ual` หรือโดย
  หยุด User Access Logging Service) และฐานข้อมูลที่อยู่ในสถานะ dirty จะขาดการเปลี่ยนแปลงล่าสุดของตน
- หากไม่มี `SystemIdentity.mdb` ค่า `RoleName` จะมาจากชื่อที่ Windows Server ตั้งให้บทบาทของตน

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

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

## ตารางเอาต์พุต { #output-tables }

CSV จะสร้างหนึ่งไฟล์ต่อหนึ่งตาราง โดยมีเพียงแถวส่วนหัวเมื่อตารางนั้นว่าง แถว JSON และ JSONL
มี `SourceFile` (ฐานข้อมูล) และ `Table` (ชื่อของตารางในฐานข้อมูล)

| ไฟล์ CSV | ตาราง | เนื้อหา |
|---|---|---|
| `UAL-Clients.csv` | `CLIENTS` | ไคลเอนต์ของแต่ละบทบาท: แอดเดรส บัญชี การเข้าถึงครั้งแรกและครั้งล่าสุดของปี จำนวนคำขอรวมและรายวัน |
| `UAL-RoleAccess.csv` | `ROLE_ACCESS` | เวลาที่แต่ละบทบาทถูกใช้งานครั้งแรกและครั้งล่าสุด |
| `UAL-DNS.csv` | `DNS` | คู่แอดเดรสกับชื่อโฮสต์ พร้อมเวลาที่พบแต่ละคู่ครั้งล่าสุด |
| `UAL-VirtualMachines.csv` | `VIRTUALMACHINES` | เครื่องเสมือน Hyper-V: ตัวระบุ หมายเลขซีเรียล เวลาที่สร้าง และเวลาที่ทำงานอยู่ครั้งล่าสุด |
| `UAL-SystemIdentity.csv` | `SYSTEM_IDENTITY` | ตัวเซิร์ฟเวอร์: ชื่อ โดเมน เวอร์ชัน Windows ประเภทผลิตภัณฑ์ ฮาร์ดแวร์ เขตเวลา และการบูตครั้งล่าสุด แถวใหม่อาจแสดงถึงการเปลี่ยนชื่อ การเข้าร่วมโดเมน หรือการเลื่อนระดับเป็น domain controller |
| `UAL-ChainedDatabases.csv` | `CHAINED_DATABASES` | ไฟล์ `{GUID}.mdb` ของแต่ละปี |
| `UAL-Roles.csv` | `ROLE_IDS` | GUID ของบทบาทพร้อมชื่อ |
| `UAL-<table>.csv` | ตารางอื่น | ทุกคอลัมน์ของตารางที่ Juntei ไม่รู้จัก |

## ฟิลด์ { #fields }

| ฟิลด์ | การตีความ |
|---|---|
| `SourceFile` | ฐานข้อมูลที่อ่านแถวนั้นมา |
| `RoleGUID`, `RoleName` | บทบาท และชื่อของบทบาทจาก `SystemIdentity.mdb` ที่อยู่ข้างฐานข้อมูล (ใช้ชื่อในตัวหากไม่มีไฟล์นั้น) |
| `TenantID` | GUID ของ tenant ของไคลเอนต์ เมื่อบทบาทรายงานค่านี้ |
| `TotalAccesses` | จำนวนคำขอของไคลเอนต์ในปีของฐานข้อมูล เท่ากับผลรวมของ `DailyAccesses` |
| `InsertDate` | เวลาที่สร้างแถว: คำขอแรกของไคลเอนต์ในปีนั้นในฐานข้อมูลนี้ (UTC) |
| `LastAccess` | คำขอล่าสุดของไคลเอนต์ (UTC) |
| `Address` | แอดเดรส IPv4 หรือ IPv6 ของไคลเอนต์ |
| `AuthenticatedUserName` | บัญชีที่ไคลเอนต์ใช้ยืนยันตัวตน (`DOMAIN\user` โดยบัญชีคอมพิวเตอร์ลงท้ายด้วย `$`) |
| `ClientName` | ชื่อของไคลเอนต์ เมื่อบทบาทรายงานค่านี้ |
| `DailyAccesses` | วันที่มีคำขอพร้อมจำนวน จากคอลัมน์ `Day1` … `Day366` โดยระบุวันที่ตามปีของ `LastAccess`: `2023-01-12: 5, 2023-01-13: 2` ใน CSV และอ็อบเจกต์ที่มี `Date` และ `Accesses` ใน JSON วันที่ไม่ทราบปีจะมี `DayOfYear` แทน `Date` |
| `FirstSeen`, `LastSeen` | การใช้งานบทบาทครั้งแรกและครั้งล่าสุด (`ROLE_ACCESS`) และการพบคู่ DNS ครั้งล่าสุด (`DNS`) เป็นเวลา UTC |
| `CreationTime`, `LastSeenActive` | เวลาที่สร้างเครื่องเสมือนหรือแถว `SYSTEM_IDENTITY` และเวลาที่เครื่องเสมือนทำงานอยู่ครั้งล่าสุด เป็นเวลา UTC |
| `OS-ProductType`, `OS-ProductTypeName` | 1 `Workstation`, 2 `Domain Controller`, 3 `Server` |
| `OS-CurrentTimeZone` | ค่าชดเชยเวลาของเซิร์ฟเวอร์จาก UTC เป็นนาที (`-420` คือ UTC-07:00) |
| `OS-LastBootUpTime` | การบูตครั้งล่าสุดของเซิร์ฟเวอร์เป็นเวลา UTC (เป็นข้อความ WMI ตามที่จัดเก็บไว้เมื่ออ่านค่าไม่ได้) |
| คอลัมน์อื่น | ทุกคอลัมน์อื่นของตารางภายใต้ป้ายชื่อฟิลด์แบบ PascalCase ของ Juntei เช่น `SystemDNS-HostName`, `SystemDomainName`, `OS-BuildNumber` และ `SystemSerialNumber` |

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียนตารางลงใน `LateralMovement` เป็น `UAL-Clients.csv`, `UAL-RoleAccess.csv`
และอื่น ๆ (เป็นไฟล์ `UAL.json` หรือ `UAL.jsonl` ไฟล์เดียวเมื่อใช้ `-t json` หรือ `-t jsonl`) พร้อม
`LateralMovement/UAL-logs.txt` ไทม์ไลน์แบบรวม `-j` จะรวมทุกแถว พร้อม `Comp` และ
ตัวระบุ `Artifact` ที่เป็น `ual` ส่วน `-Z` จะแปลง `InsertDate`, `LastAccess`, `FirstSeen`, `LastSeen`,
`CreationTime`, `LastSeenActive` และ `OS-LastBootUpTime`
