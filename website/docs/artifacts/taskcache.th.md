# แคช Task Scheduler { #task-scheduler-cache }

## TaskCache คืออะไร { #what-taskcache-is }

Windows Task Scheduler เก็บแคชในรีจิสทรีไว้ใต้ `SOFTWARE` ซึ่งเชื่อมโยงพาธของงาน (task) กับ GUID,
trigger/รายการ principal การดำเนินการ (action) เมทาดาทาขณะทำงาน และ security descriptor Juntei ถอดรหัส
action แบบการเรียกใช้งานและ COM handler รวมถึง action แบบอีเมล/ข้อความแบบเดิม timestamp ของงาน และสัญญาณของ
งานที่ถูกซ่อนจากอินเทอร์เฟซ Task Scheduler ตามปกติ

งานที่ตั้งเวลาไว้เป็นกลไกที่ใช้กันทั่วไปสำหรับการคงอยู่และการเรียกใช้งานระยะไกล แคชในรีจิสทรียังมีคุณค่า
เมื่อผู้โจมตีลบหรือดัดแปลงไฟล์ XML ของงานด้วย

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> TaskCache ได้กำหนดค่าหรือเก็บรักษา
งาน/action นั้นไว้ จำเป็นต้องมีฟิลด์ขณะทำงานและ log อิสระเพื่อพิสูจน์การเรียกใช้งานครั้งใดครั้งหนึ่ง

## ตำแหน่ง { #locations }

```text
SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache
C:\Windows\System32\Tasks\...   (task XML/files; collect separately)
```

ให้เก็บรวบรวม `SOFTWARE` พร้อม transaction log และโครงสร้าง `System32\Tasks` ทั้งหมด event log
Operational ของ Task Scheduler และ Security ช่วยเพิ่มหลักฐานการสร้าง/การอัปเดต/การทำงาน

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- แคชในรีจิสทรีและ XML ของงานอาจไม่ตรงกันหลังการดัดแปลง การลบ การลงทะเบียนที่ไม่สมบูรณ์ หรือความเสียหาย
- `Hidden` ของ Juntei ระบุการซ่อนเชิงโครงสร้าง (ไม่มีการเชื่อมโยงใน Tree/security descriptor ภายใต้
  เงื่อนไขที่เกี่ยวข้อง) ไม่ใช่เพียงงานที่ตั้งค่าแฟล็ก “Hidden” ตามปกติของ UI
- Windows 7 จัดเก็บ action ของงานไว้ใน XML แทนที่จะเป็นค่า `Actions` ของแคช action ที่ว่างเปล่า
  บนระบบนั้นจึงไม่ได้น่าสงสัยโดยอัตโนมัติ
- `LastRun` และ `LastResult` มีประโยชน์ แต่ไม่ใช่ประวัติการเรียกใช้งานที่สมบูรณ์และป้องกันการดัดแปลงได้
- `Context` ระบุ principal/context ที่กำหนดค่าไว้ ไม่ใช่ผู้ใช้ที่ลงทะเบียนงาน
- CLSID ของ COM handler ต้องแปลงค่าแยกต่างหากผ่านรีจิสทรี/ไฟล์

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry taskcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE hives
  -f, --file <FILE>      Parse one SOFTWARE hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --task <NAME>  Full task path or final task name

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry taskcache -f SOFTWARE -o tasks.csv
juntei parse-registry taskcache -f SOFTWARE -t jsonl \
  | jq 'select(.Hidden == true or (.Executable? | test("powershell|cmd|mshta"; "i")))'
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | บริบทของโฮสต์และไฮฟ์ `SOFTWARE` ต้นทาง |
| `TaskName` | พาธเต็มใน scheduler เป็นจุดเชื่อมโยงหลักไปยัง XML และ event log |
| `TaskGUID` | GUID ที่เชื่อม entry ของแคช Tree และ Tasks เป็นคีย์เชื่อมโยงที่แข็งแกร่ง |
| `TaskIndex` | เมทาดาทา Tree/index ที่ scheduler ใช้ เป็นข้อมูลบริบท ไม่ใช่จำนวนครั้งที่เรียกใช้ |
| `TaskLists` | รายการในแคช (`Boot`, `Logon`, `Plain`, `Maintenance`) ที่จัดหมวดหมู่การทริกเกอร์/การจัดการ |
| `Hidden` | ตัวบ่งชี้งานที่ถูกซ่อนเชิงโครงสร้าง ให้ตรวจทาน `InTree`, `InTasks` และสถานะของ descriptor ก่อนสรุปว่ามีการดัดแปลง |
| `InTree`, `InTasks` | มีแคชทั้งสองฝั่งหรือไม่ การเชื่อมโยงที่ขาดหายอาจบ่งชี้การลบ/การดัดแปลง/การลงทะเบียนที่ไม่สมบูรณ์ |
| `HasSecurityDescriptor` | entry ใน Tree มี `SD` หรือไม่ การไม่มีอาจมีส่วนทำให้เกิดพฤติกรรมงานที่ถูกซ่อนในเวอร์ชันที่เกี่ยวข้อง |
| `TreeKeyLastWrittenUTC`, `TaskKeyLastWrittenUTC` | การเปลี่ยนแปลงระดับคีย์ของแคชแต่ละฝั่ง ไม่ใช่เวลาสร้าง/เวลาทำงานที่แน่นอน |
| `ActionIndex` | action ภายในงาน นับจากศูนย์ งานหนึ่งอาจสร้างหลายแถว |
| `ActionType` | `exec`, `com_handler`, `send_email` หรือ `show_message` |
| `Executable`, `Args`, `WorkingDirectory` | action การเรียกใช้งานที่มีผลจริง ให้ตีความเครื่องหมายคำพูด ตัวแปร พาธสัมพัทธ์ และ LOLBin |
| `COM-ClassID`, `ActionData` | CLSID และข้อมูลของ COM handler หรือข้อมูลอีเมล/ข้อความแบบเดิม ให้แปลง CLSID แยกต่างหาก |
| `Context` | principal context ที่เข้ารหัสไว้กับ action (`LocalSystem`, `Author` เป็นต้น) เป็นตัวตนที่กำหนดค่าไว้ ไม่ใช่ผู้ลงทะเบียน |
| `Author`, `URI`, `Description` | เมทาดาทาของงาน ผู้โจมตีควบคุมได้ง่ายและไม่ใช่หลักฐานความน่าเชื่อถือ |
| `LastRegisteredUTC` | เวลาลงทะเบียน/อัปเดตจาก DynamicInfo เมื่อมีอยู่ |
| `LastRunUTC`, `LastSuccessfulRunUTC`, `LastResult` | เมทาดาทาการทำงาน/ผลลัพธ์ที่แคชไว้ ให้เชื่อมโยงกับ operational log และหลักฐานของโพรเซส |
| `ActionsRaw` | ไบต์ action ต้นฉบับ หากการถอดรหัสไม่สมบูรณ์หรือจำเป็นต้องตรวจสอบ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | พาธ/GUID ไม่ตรงกัน ชนิดของค่าไม่ถูกต้อง โครงสร้าง action ถูกตัดทอน หรือส่วนประกอบที่ขาดหาย |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ตรวจทานงานที่ถูกซ่อน/ไม่สอดคล้องกันและพาธที่ไม่ใช่ของ Microsoft ก่อน
2. สร้างคำสั่งที่มีผลจริงขึ้นใหม่ รวมถึงอาร์กิวเมนต์ ไดเรกทอรีทำงาน การขยายตัวแปรสภาพแวดล้อม และการแปลง COM
3. เปรียบเทียบระเบียนในรีจิสทรีกับ XML ของงาน และคำนวณแฮช/ตรวจสอบ timestamp ของไฟล์ปฏิบัติการที่อ้างอิงถึง
4. เชื่อมโยง `TaskName`/GUID/เวลา กับเหตุการณ์ของ Task Scheduler, Security 4698/4702, log ของโพรเซส, Prefetch และ MFT/USN
5. ค้นหาข้อมูลรีจิสทรีที่ถูกลบ/กู้คืนได้เมื่อผู้โจมตีลบงานออกไป

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: สคีมาของ Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-schema)
- [Microsoft: event log ของ Task Scheduler](https://learn.microsoft.com/en-us/windows/win32/taskschd/task-scheduler-1-0-events)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [MITRE ATT&CK T1053.005 Scheduled Task](https://attack.mitre.org/techniques/T1053/005/)
