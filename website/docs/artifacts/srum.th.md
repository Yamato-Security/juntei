# SRUM { #srum }

## SRUM คืออะไร { #what-srum-is }

System Resource Usage Monitor (SRUM) จัดเก็บฐานข้อมูล ESE ของการใช้ทรัพยากรโดยแอปพลิเคชันและบริการ
ขึ้นอยู่กับเวอร์ชันของ Windows ข้อมูลที่บันทึกได้แก่ จำนวนไบต์ของเครือข่าย ตัวระบุอินเทอร์เฟซ/โปรไฟล์
การเชื่อมต่อ เวลา CPU กิจกรรมแบบ foreground/background กิจกรรมดิสก์ การใช้พลังงาน
push notification และข้อมูลไทม์ไลน์ของแอปพลิเคชัน รายการต่าง ๆ จะเชื่อมโยงกับ ID ของแอปพลิเคชันและ
SID ของผู้ใช้ผ่าน `SruDbIdMapTable`

SRUM มีประโยชน์อย่างยิ่งเมื่อไฟล์ปฏิบัติการถูกลบไปแล้ว: ID ของแอปพลิเคชัน ผู้ใช้ และแถวการใช้
ทรัพยากรอาจคงอยู่นานกว่าตัวไฟล์ ยอดรวมของเครือข่ายสามารถสนับสนุนสมมติฐานการนำข้อมูลออก (exfiltration) ได้ แต่
SRUM ไม่ใช่การดักจับแพ็กเก็ต และโดยทั่วไปไม่ได้ระบุปลายทางระยะไกลหรือไฟล์ที่ถูกถ่ายโอนทุกรายการ

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Windows ระบุว่าการใช้ทรัพยากรที่บันทึกไว้
เป็นของแอปพลิเคชัน/บัญชีนั้นในช่วงเวลาการวัดของตาราง

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\Windows\System32\sru\SRUDB.dat
C:\Windows\System32\config\SOFTWARE   (context for account and Wi-Fi names)
```

SRUM มีอยู่บน Windows 8 และรุ่นใหม่กว่า ให้คัดลอก `SRUDB.dat` ในสถานะที่สอดคล้องกัน โดยควรคัดลอกจากสแนปช็อตหรือใช้
เครื่องมือเก็บรวบรวมทางนิติวิทยาศาสตร์ และให้เก็บรวบรวม `SOFTWARE` รวมถึงไฟล์ธุรกรรม/บันทึกของ ESE ในไดเรกทอรี `sru`
ด้วย

ไฮฟ์ SOFTWARE ที่ตรงกับระบบเป็นคนละส่วนกับฐานข้อมูล ESE เมื่อไฮฟ์มีสถานะ dirty Juntei จะเล่นซ้ำบันทึกธุรกรรม Registry ข้างเคียง `SOFTWARE.LOG`, `.LOG1` และ `.LOG2` ในหน่วยความจำโดยอัตโนมัติ จึงควรเก็บไฟล์เหล่านี้ด้วย เครื่องมือจะแสดงรายละเอียดการกู้คืนและคำเตือน หากไม่มีบันทึก ชื่อบัญชีหรือ Wi-Fi อาจไม่ครบหรือล้าสมัย การทำงานนี้ไม่ได้เล่นซ้ำบันทึกของฐานข้อมูล SRUM

!!! warning "ฐานข้อมูลที่อยู่ในสถานะ dirty"
    Juntei อ่าน `SRUDB.dat` ที่อยู่ในสถานะ dirty ตามที่เก็บรวบรวมมาและรายงานสถานะนั้น ปัจจุบัน Juntei ยังไม่เล่นซ้ำ
    `SRU*.log` ดังนั้นการเปลี่ยนแปลงที่มีอยู่เฉพาะในบันทึกเหล่านั้นจะขาดหายไป อย่าอธิบายฐานข้อมูลที่อยู่ในสถานะ
    dirty ว่าสมบูรณ์โดยไม่แจ้งข้อจำกัดนี้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- แถวจำนวนมากเป็นการรวมข้อมูลเป็นระยะ ซึ่งโดยทั่วไปอยู่ที่ขอบเขตรายชั่วโมง ไม่ใช่เวลาที่เกิดการกระทำจริง
- ยอดรวมไบต์เพียงอย่างเดียวไม่ได้ระบุเนื้อหา เจตนา โปรโตคอล หรือปลายทาง
- ตัวตนของแอปพลิเคชันอาจเป็นพาธ บริการ แอปแบบแพ็กเกจ หรืออ็อบเจกต์ ให้ตรวจสอบ `AppType`
- ชื่อผู้ใช้และชื่อ Wi-Fi เป็นข้อมูลเสริมจากไฮฟ์ `SOFTWARE` ที่ตรงกัน การไม่มีชื่อไม่ได้หมายความว่า
  ID ที่อยู่เบื้องหลังไม่มีอยู่
- `null` ศูนย์ คอลัมน์ที่ไม่มีอยู่ ชนิดที่ไม่รองรับ และค่าที่เสียหาย เป็นสถานะที่แตกต่างกัน
- schema ของ provider แตกต่างกันตามเวอร์ชันของ Windows ให้เก็บรักษาตารางที่ไม่รู้จักและชื่อคอลัมน์ดั้งเดิมไว้
- บริการ Windows ที่ทำงานเบื้องหลังสร้างยอดรวมการใช้ทรัพยากรจำนวนมากได้ตามปกติ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese srum <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SRUDB.dat
  -f, --file <FILE>      Parse one SRUDB.dat
  -r, --software <FILE>  Matching SOFTWARE hive for names

Output:
  -o, --output <PATH>        Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>   csv, json, or jsonl
      --include-value-state  Add per-value read state and raw bytes to JSON/JSONL
```

```bash
juntei parse-ese srum -f SRUDB.dat -r SOFTWARE -o srum-csv
juntei parse-ese srum -f SRUDB.dat -t jsonl \
  | jq 'select(.Provider == "NetworkUsages" and (.BytesSent // 0) > 100000000)'
juntei parse-ese srum -f SRUDB.dat -t jsonl --include-value-state \
  | jq '{Provider, Timestamp, ValueState, Raw}'
```

## ตารางเอาต์พุต { #output-tables }

CSV จะสร้าง `SRUM-IdentityMap.csv` สำหรับ `SruDbIdMapTable` และหนึ่งตารางต่อหนึ่ง provider ชื่อเอาต์พุตที่รู้จักได้แก่

| Provider/ตาราง | วัตถุประสงค์ในการตรวจสอบ |
|---|---|
| `NetworkUsages` | ไบต์ที่ส่ง/รับต่อแอปพลิเคชัน ผู้ใช้ อินเทอร์เฟซ และโปรไฟล์ |
| `NetworkConnections` | กิจกรรมแบบจำนวน/ระยะเวลาการเชื่อมต่อ และเวลาเริ่มการเชื่อมต่อ |
| `AppResourceUseInfo` | การใช้ CPU ดิสก์ เครือข่ายแบบ foreground/background และการใช้งานแอปพลิเคชันที่เกี่ยวข้อง |
| `AppTimelineProvider` | กิจกรรมไทม์ไลน์ของแอปพลิเคชันพร้อมฟิลด์เวลาสิ้นสุดและสถานะการใช้งาน |
| `EnergyUsage` | ค่าการวัดพลังงาน/provider ระยะสั้นและระยะยาว `IsLt` ใช้แยกแยะทั้งสอง |
| `PushNotifications` | กิจกรรม push notification |
| `VfuProvider` | provider `vfuprov` วัตถุประสงค์โดยละเอียดไม่มีเอกสารที่ดีนัก จึงควรตีความฟิลด์อย่างระมัดระวัง |
| `EnergyEstimation`, `TaggedEnergy` | provider ด้านพลังงานและการใช้งานที่ขึ้นกับเวอร์ชัน |
| `ProcessorTime`, `SizeInBytes`, `TotalUsed`, `NetworkBytes` | ตาราง provider ของ Windows Server |
| ชื่อตารางที่ไม่รู้จัก | Juntei เก็บรักษา provider ที่ไม่รู้จักไว้ภายใต้ GUID/ชื่อตาราง และส่งออกทุกคอลัมน์ใน catalog |

เนื่องจาก SRUM ขับเคลื่อนด้วย catalog และ Windows เพิ่มคอลัมน์ของ provider อยู่เสมอ จึงจงใจไม่มีรายการ
ฟิลด์เฉพาะของ provider ที่ตายตัว Juntei ส่งออก **ทุกคอลัมน์ที่มีอยู่ในฐานข้อมูลโดยใช้
ป้ายชื่อฟิลด์แบบ PascalCase ของ Juntei** ฟิลด์ต่อไปนี้เป็นข้อตกลงเอาต์พุตที่คงที่/ใช้ร่วมกัน

## เอาต์พุตของ auto-parse ที่จัดหมวดหมู่ { #categorized-auto-parse-output }

`auto-parse` วางตาราง SRUM แต่ละตารางไว้ในหมวดหมู่การตรวจสอบของตารางนั้นโดยตรง CSV, JSON และ JSONL
ใช้คำนำหน้าชื่อไฟล์ `SRUM-` เหมือนกัน โดยไม่มีไดเรกทอรี `SRUM/` ครอบอยู่

| หมวดหมู่ | ไฟล์ CSV |
|---|---|
| `NetworkActivity` | `SRUM-NetworkConnections.csv`, `SRUM-NetworkUsages.csv`, `SRUM-NetworkBytes.csv`, `SRUM-PushNotifications.csv` |
| `ProcessExecution` | `SRUM-AppResourceUseInfo.csv`, `SRUM-AppTimelineProvider.csv`, `SRUM-ProcessorTime.csv` |
| `SystemInformation` | `SRUM-EnergyUsage.csv`, `SRUM-EnergyEstimation.csv`, `SRUM-TaggedEnergy.csv`, `SRUM-SizeInBytes.csv`, `SRUM-TotalUsed.csv`, `SRUM-IdentityMap.csv`, `SRUM-VfuProvider.csv` และตารางของ provider ที่ไม่รู้จัก |

จะส่งออกเฉพาะตารางที่มีอยู่ในข้อมูลที่เก็บรวบรวมมาเท่านั้น ตัวนับเครือข่ายและเรคคอร์ดการแจ้งเตือน
ถูกจัดวางตามการใช้งานหลักในการตรวจสอบ ซึ่งไม่ได้หมายความว่าทุกแถวอธิบายการเชื่อมต่อ
ระยะไกล identity map รองรับหลายหมวดหมู่ ส่วน VFU และ provider ที่ไม่รู้จักจะอยู่ใน
`SystemInformation` เนื่องจากการมีอยู่เพียงลำพังไม่ได้ยืนยันการเรียกใช้งาน

`SystemInformation/SRUM-logs.txt` มีข้อมูลวินิจฉัยของพาร์เซอร์และการจับคู่ชื่อเอาต์พุตดั้งเดิมกับชื่อสุดท้าย
manifest ทั้งสองแสดงพาธของตารางจริงใน `OutputFiles` ส่วน `Output` ของงาน SRUM
คือไดเรกทอรีของคอมพิวเตอร์ และ `Category` ระบุตำแหน่งบันทึก/เมทาดาทาที่ใช้ร่วมกัน

เมื่อมีหลายฐานข้อมูล ชื่อไฟล์จะมี `-Source1`, `-Source2` ฯลฯ เพิ่มเข้ามาก่อนนามสกุล
บันทึกจะเก็บรักษาชื่อแทน (alias) ของข้อมูลที่เก็บรวบรวมดั้งเดิมไว้ และแถว JSON/JSONL จะเก็บแหล่งที่มา `SourceFile`,
`Table`, `Provider`, `Comp` และ `Artifact` ไว้ ไม่มีตารางใดถูกคัดลอกไปยังหลาย
หมวดหมู่ ไทม์ไลน์ SIEM แบบรวม `-j` จะรวมเรคคอร์ด SRUM ที่เก็บไว้ทุกรายการเพียงครั้งเดียว

`parse-ese srum` แบบเรียกใช้เดี่ยวจะใช้ตำแหน่งเอาต์พุตตามที่ร้องขอ ไฟล์ตาราง CSV ของคำสั่งนี้ก็มี
คำนำหน้า `SRUM-` เช่นกัน ส่วนชื่อไฟล์เอาต์พุต JSON/JSONL ที่ระบุอย่างชัดเจนยังคงเป็นไปตามที่นักวิเคราะห์เลือก

## ฟิลด์เอาต์พุตทั่วไป { #common-output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `SourceFile` | ฐานข้อมูลอินพุต มีอยู่ใน JSON/JSONL และมีประโยชน์เมื่อทำงานข้ามหลายโฮสต์ |
| `Table` | ชื่อ/GUID ดั้งเดิมของตาราง ESE ให้เก็บรักษาไว้เมื่อเปรียบเทียบเวอร์ชันหรือ provider ที่ไม่รู้จัก |
| `Provider` | ชื่อ provider/เอาต์พุตที่อ่านง่าย หรือชื่อตารางดั้งเดิมหากไม่รู้จัก |
| `ID` | `AutoIncId` ของแถว เป็นตัวระบุแถวของฐานข้อมูล ไม่ใช่ process ID |
| `Timestamp` | เวลาที่ SRUM บันทึก/รวมตัวเลข ปัดเป็นมิลลิวินาที โดยปกติไม่ใช่เวลาที่เกิดเหตุการณ์จริง |
| `EXE-Info` | ตัวตนของแอปพลิเคชัน/บริการ/แพ็กเกจที่ resolve จาก `AppID` เป็นจุดเชื่อมโยงหลักของโปรแกรม |
| `EXE-InfoDescription` | คำอธิบายที่แยกออกมาจากชื่อแอปพลิเคชันแบบประกอบของ SRUM ใช้เป็นบริบทเท่านั้น |
| `EXE-Timestamp` | เวลาที่ฝังอยู่ในตัวตนของแอปพลิเคชันแบบประกอบ (ถ้ามี) ไม่ใช่เวลาการใช้งานของแถว |
| `AppType` | `NormalApp`, `Service`, `ModernApp` หรือ `Object` เป็นตัวกำหนดวิธีตีความ `EXE-Info` |
| `SID` | SID ของบัญชีที่ resolve จาก `UserID` เป็นจุดเชื่อมโยงตัวตนของผู้ใช้/บริการที่แข็งแรง |
| `User` | ชื่อโปรไฟล์ที่อ่านง่ายซึ่งเสริมจาก `SOFTWARE` อาจเป็น null ขณะที่ `SID` ยังคงถูกต้อง |
| `UserID`, `AppID` | reference ตัวเลขดิบไปยัง `SruDbIdMapTable` ให้เก็บรักษาไว้สำหรับชื่อที่ resolve ไม่ได้หรือซ้ำกัน |
| `IsLt` | `true` สำหรับตาราง EnergyUsage ระยะยาว และ `false` สำหรับระยะสั้น |
| `InterfaceLUID` | LUID ดิบของอินเทอร์เฟซเครือข่ายจากแถวของ provider |
| `InterfaceType` | คลาสอินเทอร์เฟซที่ถอดรหัสแล้ว (เช่น `IF_TYPE_IEEE80211`) เป็นคลาส ไม่ใช่อะแดปเตอร์ที่ไม่ซ้ำกัน |
| `L2ProfileID` | reference ตัวเลขของโปรไฟล์เครือข่าย |
| `ProfileName` | ชื่อ Wi-Fi/โปรไฟล์ที่เสริมจาก `SOFTWARE` อาจเป็นข้อมูลอ่อนไหว และไม่ใช่หลักฐานของการเชื่อมต่อปัจจุบัน |
| `ConnectStartTime`, `EventTimestamp`, `EndTime`, `StartTime` | คอลัมน์ FILETIME ของ provider ที่รู้จักซึ่งถอดรหัสเป็น UTC ความหมายที่แน่นอนขึ้นกับ provider ไม่ใช่เวลาเรียกใช้งานที่ใช้ได้ทั่วไป |
| คอลัมน์เฉพาะของ provider | คอลัมน์ของ provider ที่ส่งออก เช่น `BytesSent`, `BytesReceived` (คอลัมน์ฐานข้อมูล `BytesRecvd`) ตัวนับ CPU/ดิสก์ ระยะเวลา จำนวน แฟล็ก หรือค่าประมาณ ให้ตีความหน่วยตาม provider และเปรียบเทียบแถวภายใน schema เดียวกัน |

## ฟิลด์วินิจฉัย (`--include-value-state`) { #diagnostic-fields-include-value-state }

| ฟิลด์ | การตีความ |
|---|---|
| `ValueState` | แมปจากฟิลด์ฐานข้อมูลไปยัง `parsed`, `null`, `absent`, `unsupported` หรือ `damaged` ใช้แยกการไม่มีหลักฐานออกจากความล้มเหลวของพาร์เซอร์ |
| `Raw` | ไบต์ดั้งเดิมในรูปเลขฐานสิบหกสำหรับค่าที่มีไบต์ ใช้สำหรับการตรวจสอบความถูกต้องและการถอดรหัสในอนาคต |

`SRUM-IdentityMap.csv` (จาก `SruDbIdMapTable`) มี `IdIndex`, `IdType` และ `IdBlob` ได้แก่ ดัชนีดิบ ประเภทของตัวตน
และข้อความโปรแกรม/SID/อ็อบเจกต์ที่ถอดรหัสแล้วซึ่งแถวของ provider ใช้อ้างอิง

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ระบุโปรแกรมด้วย `EXE-Info`, `AppType` และ `AppID` อย่าสับสน `Id` กับ PID
2. จัดกลุ่มแถวเครือข่ายตามโปรแกรม ผู้ใช้ โปรไฟล์ อินเทอร์เฟซ และช่วงเวลา
3. เปรียบเทียบไบต์ที่ส่ง/รับกับ baseline ปกติและโฮสต์อื่น
4. เชื่อมโยงช่วงเวลาที่น่าสงสัยกับ Prefetch, event log, ข้อมูล DNS/proxy/ไฟร์วอลล์ และการเข้าถึงไฟล์
5. รายงานสถานะฐานข้อมูลที่อยู่ในสถานะ dirty และการไม่ได้เล่นซ้ำบันทึกธุรกรรมเป็นข้อจำกัดของการเก็บรวบรวม

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser)
- [SrumECmd โดย Eric Zimmerman](https://github.com/EricZimmerman/Srum)
- [dissect.esedb](https://github.com/fox-it/dissect.esedb)
- [libyal libesedb](https://github.com/libyal/libesedb)
- [Microsoft: Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
