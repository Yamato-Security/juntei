# บริการ Windows { #windows-services }

## การกำหนดค่าบริการบันทึกอะไรไว้ { #what-service-configuration-records }

บริการ Windows และไดรเวอร์เคอร์เนลถูกกำหนดค่าไว้ใต้ control set แต่ละชุดของ `SYSTEM` เรคคอร์ดจะระบุ
ไฟล์ปฏิบัติการ/ไดรเวอร์หรือ DLL ของบริการ โหมดการเริ่มทำงาน ชนิดบริการ บัญชี กลุ่มลำดับการโหลด การเริ่มแบบหน่วงเวลา
พฤติกรรมเมื่อเกิดข้อผิดพลาด และการดำเนินการที่ Windows ทำหลังจากบริการล้มเหลว

ผู้โจมตีสร้างหรือแก้ไขบริการเพื่อการคงอยู่ (persistence) การยกระดับสิทธิ์ การโหลดไดรเวอร์ และ
การเคลื่อนย้ายภายในเครือข่าย ซอฟต์แวร์ที่ถูกต้องตามกฎหมายก็ใช้กลไกเดียวกันทุกประการ ดังนั้นพาธ บัญชี การลงนาม
บริบทการสร้าง และหลักฐานขณะทำงานจึงมีความสำคัญ

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> control set ที่วิเคราะห์ได้กำหนดค่า
บริการ/ไดรเวอร์ไว้ตามที่แสดง การกำหนดค่าเพียงอย่างเดียวไม่ได้พิสูจน์ว่าบริการเริ่มทำงานได้สำเร็จ

## ตำแหน่ง { #location }

```text
SYSTEM\ControlSet00N\Services\<ServiceName>
SYSTEM\Select\Current   (which control set Windows used)
```

ให้เก็บรวบรวม `SYSTEM` พร้อม transaction log ของไฟล์นั้น Event ID เช่น System 7045 และ Security 4697 สามารถ
บันทึกการติดตั้งได้ ส่วนเหตุการณ์ควบคุมบริการและเหตุการณ์ของโพรเซสสามารถสนับสนุนการเริ่มทำงาน/การเรียกใช้จริงได้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- `ImagePath` อาจมีตัวแปรสภาพแวดล้อม เครื่องหมายคำพูด อาร์กิวเมนต์ พาธอุปกรณ์ หรือพาธที่ไม่มีเครื่องหมายคำพูด
  แต่มีช่องว่าง ให้เก็บรักษาค่าเดิมไว้ทุกประการก่อนทำให้เป็นมาตรฐาน (normalize)
- `ServiceDll` มีความสำคัญมากสำหรับบริการที่โฮสต์โดย `svchost.exe` เพราะ `ImagePath` เพียงอย่างเดียวอาจระบุเพียงโฮสต์
- `StartType=automatic` ไม่ได้พิสูจน์ว่าบริการเริ่มทำงานแล้วหรือยังคงเปิดใช้งานอยู่
- `ObjectName` คือบัญชีที่กำหนดค่าไว้ ไม่จำเป็นต้องเป็นผู้ใช้ที่ติดตั้งบริการ
- ไดรเวอร์และบริการ user-mode ใช้แฟล็ก `ServiceType` และรูปแบบพาธที่ต่างกัน
- failure action สามารถเรียกใช้คำสั่งในฐานะบัญชีของบริการหลังเกิดความล้มเหลวได้ นี่คือพื้นผิวการคงอยู่
  ที่มักถูกมองข้าม
- control set อื่นอาจเก็บรักษาการกำหนดค่าก่อนหน้าไว้ ให้ใช้ `IsCurrentControlSet`

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry services <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --service <NAME>    Only this service name
      --all-control-sets  Parse every ControlSet00N

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry services -f SYSTEM -o services.csv
juntei parse-registry services -f SYSTEM --all-control-sets -t jsonl \
  | jq 'select(.FailureCommand != null or (.ImagePath? | test("temp|public"; "i")))'
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | ตัวตนของระบบและไฮฟ์อินพุต ให้คงไว้เหมือนกันในทุกการเก็บรวบรวม |
| `ControlSet`, `IsCurrentControlSet` | control set ต้นทาง และ `Select\Current` ระบุชุดนั้นหรือไม่ ให้ใช้ชุดปัจจุบันเป็นหลักเมื่อกล่าวถึงสถานะที่ใช้งานอยู่ |
| `ServiceName` | ชื่อคีย์รีจิสทรี/ชื่อบริการภายใน เป็นจุดเชื่อมโยง (pivot) ที่ดีไปยังเหตุการณ์และคำสั่งของ Service Control Manager |
| `DisplayName`, `Description` | ข้อความที่แสดงต่อผู้ใช้ อาจเป็นการอ้างอิงทรัพยากรหรือถูกผู้โจมตีควบคุม |
| `ImagePath` | คำสั่งไฟล์ปฏิบัติการ/ไดรเวอร์ตามที่กำหนดค่าไว้ ให้แยกวิเคราะห์เครื่องหมายคำพูด อาร์กิวเมนต์ ตัวแปรสภาพแวดล้อม และพาธอุปกรณ์ |
| `ServiceDLL` | DLL ที่โหลดโดยบริการโฮสต์ มักเป็น payload จริงของบริการแบบ shared-process |
| `StartType`, `StartValue` | โหมดการเริ่มทำงานแบบอ่านง่าย/แบบดิบ: boot (0), system (1), automatic (2), manual (3), disabled (4) |
| `DelayedAutostart` | บริการอัตโนมัติจะเริ่มหลังช่วงบูตเริ่มต้น แต่ยังสามารถใช้เพื่อการคงอยู่ได้ |
| `ServiceType`, `TypeValue` | แฟล็กแบบถอดรหัสแล้ว/แบบดิบ: ไดรเวอร์เคอร์เนล ไดรเวอร์ระบบไฟล์ own/share process บริการผู้ใช้ เป็นต้น |
| `ErrorControl` | ระดับการจัดการข้อผิดพลาดขณะบูต/เริ่มทำงาน ส่วนใหญ่เป็นบริบทเชิงปฏิบัติการ |
| `ObjectName` | บัญชีบริการหรือชื่อออบเจกต์ไดรเวอร์ที่กำหนดค่าไว้ ไม่ใช่ตัวตนของผู้ติดตั้ง |
| `Group` | กลุ่มลำดับการโหลด มีความสำคัญเป็นพิเศษสำหรับไดรเวอร์ boot/system |
| `FailureResetPeriod` | จำนวนวินาทีก่อนรีเซ็ตตัวนับความล้มเหลว `4294967295` หมายถึงไม่รีเซ็ตเลย |
| `FailureCommand` | คำสั่งที่กำหนดค่าไว้สำหรับ failure action แบบ `run_command` เป็นฟิลด์ที่มีคุณค่าสูงสำหรับการคงอยู่ |
| `FailureActions` | รายการการดำเนินการที่ถอดรหัสแล้วทั้งหมด (`restart`, `reboot`, `run_command`, `none`) พร้อมระยะหน่วงเวลา |
| `FailureActionType`, `FailureActionDelayMs` | การดำเนินการแรกที่แปลงเป็นค่าแบนเพื่อใช้กรอง ให้ตรวจสอบรายการทั้งหมดด้วย |
| `FailureActionsRaw` | ไบต์ต้นฉบับเมื่อการถอดรหัสไม่สมบูรณ์/เสียหาย |
| `KeyLastWrittenUTC` | การเปลี่ยนแปลงล่าสุดของคีย์บริการ ไม่จำเป็นต้องเป็นเวลาติดตั้ง/เริ่มทำงาน |
| `TransactionLogStatus`, `LogsApplied` | บริบทการ replay รีจิสทรี |
| `Problems` | ชนิดข้อมูลไม่ตรงกัน โครงสร้าง failure ผิดรูปแบบ หรือฟิลด์ที่ขาดหาย/อ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ตรวจทานพาธที่ไม่เป็นมาตรฐาน ไดเรกทอรีที่เขียนได้ บัญชีที่ผิดปกติ เวลาของคีย์ที่ใหม่ และคำสั่ง failure
2. ขยายตัวแปรสภาพแวดล้อมโดยใช้การกำหนดค่าของโฮสต์ที่ได้รับผลกระทบ ไม่ใช่เครื่องของนักวิเคราะห์
3. คำนวณแฮช/ตรวจสอบลายเซ็นของไฟล์ปฏิบัติการหรือ DLL จริงหากเก็บรวบรวมมาได้
4. เชื่อมโยงชื่อ/พาธของบริการกับเหตุการณ์ 7045/4697 การสร้างโพรเซส Prefetch, `$MFT` และ USN
5. เปรียบเทียบ control set ปัจจุบันและที่ไม่ใช่ปัจจุบันเพื่อหาการคงอยู่ที่ถูกเปลี่ยนแปลง/ลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: โครงสร้างรีจิสทรีสำหรับบริการอุปกรณ์และไดรเวอร์](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/hklm-system-currentcontrolset-services-registry-tree)
- [Microsoft: SERVICE_STATUS / ชนิดและสถานะของบริการ](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/ns-winsvc-service_status)
- [Microsoft: failure action ของ ChangeServiceConfig2](https://learn.microsoft.com/en-us/windows/win32/api/winsvc/nf-winsvc-changeserviceconfig2w)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
