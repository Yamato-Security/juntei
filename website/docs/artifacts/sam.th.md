# บัญชีภายในเครื่อง SAM { #sam-local-accounts }

## SAM มีข้อมูลอะไรบ้าง { #what-the-sam-contains }

ไฮฟ์ Security Account Manager (`SAM`) จัดเก็บเรคคอร์ดบัญชีภายในเครื่อง Juntei ถอดรหัสชื่อบัญชี
RID ฟิลด์โปรไฟล์/ความคิดเห็น แฟล็กควบคุมบัญชี timestamp การเข้าสู่ระบบล่าสุด/รหัสผ่าน ตัวนับ
คำใบ้รหัสผ่าน การเชื่อมโยงกับบัญชี Microsoft พาธโปรไฟล์ และการที่ Windows ซ่อนบัญชี
จาก UI การลงชื่อเข้าใช้หรือไม่

ข้อมูลนี้มีประโยชน์ในการค้นหาผู้ดูแลระบบภายในเครื่องที่สร้างขึ้นใหม่ บัญชีที่ไม่ได้ใช้งานมานานแต่ถูกเปิดใช้งาน การเปลี่ยน
รหัสผ่าน บัญชีที่ถูกซ่อน และบัญชีที่ใช้เตรียมการสำหรับการเคลื่อนย้ายภายในเครือข่าย (lateral movement)

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> SAM ภายในเครื่องแสดงบัญชีด้วย
สถานะที่บันทึกไว้ ข้อมูลนี้ไม่ใช่ประวัติการยืนยันตัวตนที่สมบูรณ์ และไม่ได้อธิบายบัญชีโดเมน

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\Windows\System32\config\SAM
C:\Windows\System32\config\SAM.LOG1
C:\Windows\System32\config\SAM.LOG2
```

ให้เก็บรวบรวมไฮฟ์และบันทึกด้วยสิทธิ์ที่เพียงพอ/การเข้าถึงข้อมูลดิบ ข้อมูลระบุตัวตนของโฮสต์สามารถเสริมได้จาก
ไฮฟ์ `SYSTEM` ที่สอดคล้องกันในชุดข้อมูลที่เก็บรวบรวม

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- คำสั่งนี้พาร์สบัญชี **ภายในเครื่อง** ไม่ใช่ผู้ใช้โดเมน Active Directory
- Juntei จงใจไม่ดึงแฮชรหัสผ่านออกมา
- RID 500 คือบัญชี Administrator ในตัวแม้จะถูกเปลี่ยนชื่อ ชื่อเพียงอย่างเดียวจึงไม่เพียงพอ
- `LastLogon` และตัวนับเป็นสถานะในรีจิสทรี ไม่ใช่ audit trail ที่สมบูรณ์ต่อเซสชัน
- `HiddenFromLogonUI` ซ่อนบัญชีจากหน้าจอลงชื่อเข้าใช้ แต่ไม่ได้ปิดใช้งานบัญชี
- `DefaultAccount` และ `WDAGUtilityAccount` ที่มีมาในตัวอาจถูกซ่อนอย่างถูกต้องตามปกติ
- แฟล็กบัญชีที่เป็น null จากค่า `F` ที่เสียหาย/หายไป ต้องไม่ถูกตีความว่าบัญชีเปิดใช้งานอยู่
- `NamesKeyLastWrittenUTC` ประมาณบริบทการลงทะเบียนชื่อบัญชีได้ แต่ไม่รับประกันว่าเป็นเวลาที่สร้างบัญชี

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry sam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SAM hives
  -f, --file <FILE>      Parse one SAM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --rid <RID>  Parse one decimal RID (example: 500)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry sam -f SAM -o accounts.csv
juntei parse-registry sam -f SAM --rid 500 -t jsonl | jq .
juntei parse-registry sam -d evidence/ -t jsonl \
  | jq 'select(.HiddenFromLogonUi == true or (.AccountFlags | index("disabled") | not))'
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | บริบทของโฮสต์และไฮฟ์ SAM ที่แน่นอน |
| `RID` | Relative Identifier จุดเชื่อมโยงตัวตนของบัญชีที่แข็งแรง 500 คือ Administrator ในตัว |
| `User` | ชื่อบัญชีภายในเครื่องปัจจุบัน อาจถูกเปลี่ยนชื่อได้ ให้เชื่อมโยงด้วย RID |
| `FullName`, `Comment`, `UserComment` | ข้อความอธิบาย ผู้โจมตีควบคุมได้ และมีประโยชน์ในการตรวจสอบการปลอมตัว (masquerading) |
| `AccountFlags` | สถานะที่ถอดรหัสแล้ว เช่น ปิดใช้งาน ถูกล็อก ไม่ต้องใช้รหัสผ่าน บัญชีปกติ รหัสผ่านไม่มีวันหมดอายุ |
| `AccountFlagsValue` | แฟล็กตัวเลขดิบสำหรับการตรวจสอบความถูกต้องและบิตที่ไม่รองรับ |
| `CountryCode`, `CodePage` | เมทาดาทา locale/code page ของบัญชี โดยปกติเป็นบริบท ไม่ใช่การระบุตัวผู้กระทำ |
| `LastLogonUTC`, `LastLogonWholeSecondsUTC` | สถานะการเข้าสู่ระบบล่าสุดเดียวกันที่ความละเอียดเต็มและแบบวินาทีเต็มเพื่อความเข้ากันได้ ไม่ใช่ประวัติที่สมบูรณ์ |
| `PasswordLastSetUTC` | เวลาเปลี่ยนรหัสผ่านที่บันทึกไว้ ให้ตรวจสอบค่าที่ใหม่หรือเก่าเกินคาด |
| `AccountExpirationUTC` | วันหมดอายุที่กำหนดไว้เมื่อมีกำหนด null อาจหมายถึงไม่มีวันหมดอายุ/ไม่ได้ตั้งค่า |
| `LastFailedLogonUTC` | การเข้าสู่ระบบภายในเครื่องที่ล้มเหลวครั้งล่าสุดที่บันทึกในสถานะ SAM ให้ยืนยันด้วยเหตุการณ์ Security |
| `FailedLogonCount`, `LogonCount` | ตัวนับสะสม/สถานะที่มีพฤติกรรมการอัปเดตเฉพาะของ Windows ใช้เป็นบริบท ไม่ใช่จำนวนเหตุการณ์ที่แน่นอน |
| `PasswordHint` | คำใบ้ที่ผู้ใช้ตั้งไว้ซึ่งเป็นข้อมูลอ่อนไหว อาจเผยรูปแบบรหัสผ่าน ให้จัดการอย่างเหมาะสม |
| `InternetUserName` | ตัวตนบัญชี Microsoft ที่เชื่อมโยงไว้ (ถ้ามี) |
| `HiddenFromLogonUI` | สถานะการซ่อนจาก UI ให้ตรวจทาน RID ที่สูงหรือบัญชีที่สร้างเองที่ไม่คาดคิด ไม่ได้หมายความว่าถูกปิดใช้งาน |
| `HomeDirectory`, `HomeDrive`, `ProfilePath`, `LogonScript` | การกำหนดค่าสภาพแวดล้อม/การเข้าสู่ระบบของบัญชีภายในเครื่อง สคริปต์/พาธ UNC ที่ผิดปกติอาจบ่งชี้การคงอยู่ |
| `KeyLastWrittenUTC` | เวลาที่คีย์บัญชีของ RID เปลี่ยนแปลง เป็นระดับคีย์ ไม่ได้เจาะจงฟิลด์ |
| `NamesKeyLastWrittenUTC` | เวลาที่คีย์จับคู่ชื่อผู้ใช้เปลี่ยนแปลง มีประโยชน์ในการประมาณวงจรชีวิตของบัญชี |
| `LastLogonFiletime`, `PasswordLastSetFiletime`, `AccountExpirationFiletime`, `LastFailedLogonFiletime` | ค่า FILETIME 64 บิตดิบที่เก็บไว้เพื่อการตรวจสอบความถูกต้องและการจัดการค่า sentinel พิเศษ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการเล่นซ้ำของรีจิสทรี |
| `Problems` | ความไม่สอดคล้องระหว่าง F/V/Names รายการที่หายไป offset ที่ผิดรูปแบบ หรือความเสียหายในการถอดรหัส |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เปรียบเทียบ RID ชื่อ แฟล็ก พาธโปรไฟล์ และ `NamesKeyLastWrittenUTC` กับ baseline ของโฮสต์
2. ตรวจทานบัญชี RID 1000 ขึ้นไปที่เปิดใช้งาน บัญชีที่ไม่ต้องใช้รหัสผ่าน รหัสผ่านไม่มีวันหมดอายุ ถูกซ่อน หรือเพิ่งเปลี่ยนแปลง
3. เชื่อมโยงกับเหตุการณ์ Security 4720/4722/4724/4732 เหตุการณ์การเข้าสู่ระบบ การสร้างโปรไฟล์ และกิจกรรมระยะไกล
4. ค้นหาประวัติการกู้คืนรีจิสทรีหากบัญชีถูกลบออกก่อนการเก็บรวบรวม
5. แยกแยะบัญชีเริ่มต้น/บัญชีที่จัดเตรียมอย่างถูกต้อง ออกจากบัญชีที่ผู้โจมตีสร้างขึ้นเพื่อใช้งานแบบโต้ตอบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft Open Specifications: SAM Remote Protocol](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-samr/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ปลั๊กอิน SAM ของ dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/sam.html)
