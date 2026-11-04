# Persistence Hunter { #persistence-hunter }

การคงอยู่ (persistence) คือวิธีทำให้โค้ดทำงานซ้ำอีกครั้ง เช่น เมื่อ Windows เริ่มทำงาน เมื่อผู้ใช้ลงชื่อเข้าใช้ เมื่อแอปพลิเคชัน
เปิดขึ้น เมื่อถึงเวลาที่กำหนด หรือเมื่อเกิดเหตุการณ์อื่น ผู้โจมตีสามารถใช้การตั้งค่าเดียวกันกับที่
ซอฟต์แวร์ที่ถูกต้องตามกฎหมายใช้สำหรับการอัปเดต การรองรับอุปกรณ์ ความปลอดภัย และการดูแลระบบ ไปในทางที่ผิดได้

`juntei persistence-hunter` ตรวจสอบหลักฐาน Windows ที่เก็บรวบรวมมาแบบออฟไลน์ด้วย **การตรวจสอบ 131 รายการ** ซึ่งครอบคลุม
กลไกเหล่านี้และความผิดปกติที่เกี่ยวข้องกับบัญชี/การป้องกัน โดยอ่านไฮฟ์รีจิสทรี ไฟล์งานตามกำหนดเวลา
สคริปต์ การตั้งค่าแอปพลิเคชัน และเรคคอร์ดฐานข้อมูลบางส่วน

หน้านี้อธิบายการตรวจสอบทุกรายการที่มีการใช้งาน สำหรับตัวเลือกทั้งหมด ฟิลด์เอาต์พุต โปรไฟล์ และ
การจัดการ timestamp ให้เปิด[คู่มือคำสั่ง](../commands/persistence-hunter.md) ส่วน
[กิตติกรรมประกาศ](acknowledgements.md) มีลิงก์ไปยังโปรเจกต์ PyrsistenceSniper ที่เขียนด้วย Python และเครื่องมือ
PersistenceSniper ต้นฉบับที่เขียนด้วย PowerShell

## เริ่มการตรวจสอบ { #start-an-investigation }

```bash
juntei persistence-hunter -d evidence/ --min-severity info -o PersistenceHunter.csv --coverage Coverage.csv
```

ใช้ `--min-severity info` เพื่อให้ได้รายการผู้ต้องสงสัย (candidate) ทั้งหมด คำสั่งที่เรียกใช้ด้วยตนเองมีค่าเริ่มต้นเป็น
`medium` โปรไฟล์ของคำสั่งอาจจัดผลที่ตรงกับรายการอนุญาตเพียงบางส่วนเป็น `LOW` แม้ว่าการขาดหลักฐานของ payload หรือ
ผู้ลงนามจะทำให้เปรียบเทียบได้ไม่ครบถ้วน ทั้งการตรงกับรายการอนุญาตและรายการที่ไม่คุ้นเคยต่างก็ไม่ได้ชี้ขาด
ว่าสิ่งใดเป็นอันตราย `auto-parse` เก็บผลทุกระดับความรุนแรงไว้อยู่แล้ว และเขียนผลการตรวจพบ
ไว้ในหมวด `persistence/` ของแต่ละคอมพิวเตอร์

หากต้องการเจาะจงที่กลไกใดกลไกหนึ่ง ให้คัดลอก **Check ID** ของกลไกนั้นจากด้านล่าง:

```bash
juntei persistence-hunter --list-checks
juntei persistence-hunter -d evidence/ --check run_keys,scheduled_task_files --min-severity info -o focused.csv
```

โดยค่าเริ่มต้น คำสั่งนี้ใช้ worker pool ร่วมกัน เพิ่ม `-T 4` เพื่อเลือกใช้ worker สี่ตัว หรือ `-T 1`
สำหรับการสแกนแบบเธรดเดียว อินพุตอาจมีหลายคอมพิวเตอร์ได้ โดยหลักฐานของแต่ละเครื่องจะถูกแยกจากกัน

## อ่านผลการตรวจพบในฐานะหลักฐาน { #read-findings-as-evidence }

| คำถาม | สิ่งที่ต้องตรวจสอบ |
|---|---|
| มีการกำหนดค่าอะไร และที่ไหน? | `CheckId`, `Path` และ `Value` ฉบับเต็ม รวมถึงอาร์กิวเมนต์ ติดตาม `SourceFile` ไปยังไฮฟ์ ไฟล์ หรือฐานข้อมูลที่เก็บรวบรวมมา |
| เป้าหมายมีอยู่ในข้อมูลที่เก็บรวบรวมหรือไม่? | `ResolvedPath`, `ResolutionStatus` และ `Exists` การอ้างอิงที่ไม่ได้เก็บรวบรวม คลุมเครือ หรือไม่รองรับ ถือว่าไม่ทราบ Juntei จะไม่ใช้ไฟล์จากเครื่องของนักวิเคราะห์มาแทน |
| การกำหนดค่ามีผลจริงหรือไม่? | ทริกเกอร์จริง แฟล็กการเปิดใช้งาน control set ที่เลือก เวอร์ชันของ Windows/แอปพลิเคชัน ผู้ใช้ และสถาปัตยกรรม การตรวจสอบบางรายการตั้งใจเก็บการกำหนดค่าที่ไม่ได้ใช้งานไว้ด้วย |
| นี่คือการโจมตีหรือไม่? | `Severity` และ `PolicyReason` อธิบายการจับคู่กับโปรไฟล์ ไม่ใช่คำตัดสิน ให้เปรียบเทียบกับซอฟต์แวร์ที่คาดว่าจะมี คำสั่งฉบับเต็ม แฮชของไฟล์ หลักฐานการเรียกใช้งานที่เป็นอิสระ และไทม์ไลน์แวดล้อม |
| ใครสามารถเรียกใช้ได้? | `AccessGained` เป็นคำใบ้ด้านขอบเขต/บริบท การกำหนดค่าระดับเครื่องมักระบุ `SYSTEM` แต่ไม่ได้พิสูจน์ว่าโพรเซสที่ถูกเรียกใช้ทำงานในฐานะ SYSTEM principal ของงานตามกำหนดเวลาและ SID เจ้าของของ BITS ให้บริบทที่เจาะจงกว่าเมื่อมีข้อมูล |
| มีการเปลี่ยนแปลงเมื่อใด? | `KeyLastWrite` ระบุเวลาของคีย์รีจิสทรีทั้งคีย์ `LastChange`, `ChangeSource` และ `ChangeEvidence` อธิบายการเชื่อมโยงกับ MFT/EVTX ที่แยกต่างหาก เมื่อมีข้อมูล ทั้งสองฟิลด์ไม่ได้ระบุเวลาการเรียกใช้ payload โดยอัตโนมัติ |
| สถานะที่กู้คืนมาเชื่อถือได้เพียงใด? | อ่าน `RecordStatus`, `Coverage.csv` และคำเตือนเกี่ยวกับหลักฐานที่ผ่านการ replay, carve, เสียหาย, ขาดหาย หรือมีข้อจำกัด เรคคอร์ดในอดีตที่กู้คืนมาอาจไม่ได้ใช้งานอยู่แล้ว |
| ผู้เผยแพร่ที่ระบุชื่อน่าเชื่อถือหรือไม่? | `Signer` และ `SignerStatus` เป็นเมทาดาทาเชิงพรรณนาของ PE/ใบรับรอง/catalog โดย Juntei ไม่ได้ตรวจสอบความถูกต้องของลายเซ็น Authenticode หรือความน่าเชื่อถือของใบรับรอง |

การไม่มีผลการตรวจพบไม่ได้พิสูจน์ว่าไม่มีการคงอยู่ แถว coverage ที่เป็น `complete` หมายความว่า
ได้อ่านแหล่งข้อมูลที่เก็บรวบรวมมาซึ่งเกี่ยวข้องแล้ว ไม่ได้หมายความว่าข้อมูลที่ได้มามีทุกไฟล์ที่เกี่ยวข้อง
ทุกผู้ใช้ หรือกลไกการคงอยู่ของ Windows ทุกรูปแบบที่เป็นไปได้ สถานะ coverage อื่น ๆ อธิบายการตรวจสอบที่ขาดหาย
ไม่ครบถ้วน ถูกปิดใช้งาน หรือล้มเหลว เครื่องมือนี้ไม่ได้สแกนหน่วยความจำขณะทำงาน เครือข่าย หรือข้อมูลประจำตัวบนคลาวด์

## แคตตาล็อกและสัญลักษณ์ระบุตำแหน่ง { #catalogue-and-location-notation }

แคตตาล็อกอ้างอิงจาก PyrsistenceSniper และเพิ่มการตรวจสอบ `start_menu_com` ที่พัฒนาขึ้นเอง
คำอธิบายด้านล่างบันทึก
พฤติกรรมของ Juntei เอง รวมถึงข้อจำกัดด้านหลักฐาน มี **การตรวจสอบที่อิงรีจิสทรี
116 รายการ และการตรวจสอบไฟล์/repository 15 รายการ** การตรวจสอบไฟล์บางรายการยังใช้ข้อมูลรีจิสทรีร่วมด้วย ตัวเลขเหล่านี้จึงเป็น
จำนวนตามการใช้งานจริง ไม่ใช่ข้อกำหนดการได้มาซึ่งข้อมูลที่แยกขาดจากกัน กลไกต่าง ๆ
จัดกลุ่มตามวัตถุประสงค์ของการสืบสวนด้านล่าง และเรียงตาม Check ID ตามลำดับตัวอักษรภายในแต่ละกลุ่ม

- `HKLM\SYSTEM`, `HKLM\SOFTWARE` และ `HKLM\SAM` หมายถึงไฮฟ์ระดับเครื่องที่เก็บรวบรวมมา ซึ่งโดยปกติ
  เก็บจาก `Windows\System32\config` พร้อม transaction log เมื่อมี
- `HKU\<user>` หมายถึงไฮฟ์ผู้ใช้ที่เก็บรวบรวมมาซึ่งเกี่ยวข้อง การตั้งค่าผู้ใช้ส่วนใหญ่มาจาก NTUSER.DAT
  ส่วนการลงทะเบียนคลาสอาจมาจาก `AppData\Local\Microsoft\Windows\UsrClass.dat` ตามที่ระบุไว้
- `<ControlSet>` หมายถึง control set ที่ใช้งานอยู่ซึ่งถูกเลือกใน SYSTEM ตำแหน่งในอดีตเพิ่มเติมของการตรวจสอบ DSRM
  ระบุไว้อย่างชัดเจนด้านล่าง ให้ตรวจสอบ `SYSTEM\Select` เมื่อตรวจทานตำแหน่งเหล่านั้น
- `<name>` คือส่วนประกอบของพาธที่เปลี่ยนแปลงได้ ในตารางเป้าหมายรีจิสทรี `<child>` หมายถึงการลงทะเบียน
  ระดับลูกโดยตรง ไม่ได้หมายถึงการไล่แบบเรียกซ้ำไม่จำกัดระดับ การตรวจสอบที่เรียกซ้ำโดยชัดแจ้ง
  จะอธิบายวิธีการไล่ไว้แยกต่างหาก วงเล็บในพาธของ Office หมายถึงเวอร์ชันที่อาจมีหรือไม่มีก็ได้
- ตารางแสดงมุมมองระดับเครื่องและระดับผู้ใช้ที่การตรวจสอบเลือกใช้จริง มุมมอง 32 บิตจะแสดง
  เมื่อมีการใช้งาน อย่าสันนิษฐานว่าการตรวจสอบทุกรายการตรวจทุกสถาปัตยกรรม/มุมมองที่เป็นไปได้
- การตรวจสอบค่าทั่วไปจะส่งออกรายการที่กำหนดค่าไว้และไม่ว่างเปล่า รวมถึงสมาชิกของ multi-string และ
  ค่าตัวเลขศูนย์เมื่อเกี่ยวข้อง การตรวจสอบเหล่านี้โดยตัวเองไม่ได้กำหนดว่าต้องมี payload ที่เป็นอันตราย
  ทริกเกอร์ที่เปิดใช้งาน หรือการเรียกใช้งานที่สำเร็จ เงื่อนไขที่เลือกเฉพาะมากกว่านี้จะระบุไว้ในแต่ละการตรวจสอบ

ATT&CK ID เป็นการจับคู่ที่สืบทอดมาจากแคตตาล็อกและเป็นลิงก์สำหรับศึกษาเพิ่มเติม ซึ่งอธิบายการใช้ในทางที่ผิดที่อาจเกิดขึ้น
ไม่ใช่ผลการตรวจพบว่าผู้โจมตีได้ใช้เทคนิคนั้น การตรวจสอบการตั้งค่าการป้องกันและการวินิจฉัย
อาจอธิบายเงื่อนไขที่เอื้อให้เกิดการโจมตี มากกว่าจะเป็นกลไกการคงอยู่ในตัวเอง

| กลุ่มกลไก | จำนวนการตรวจสอบ |
|---|---:|
| [การบูต การลงชื่อเข้าใช้ และการเริ่มทำงาน](#boot-sign-in-and-startup) | 32 |
| [บริการและส่วนขยายของระบบ](#services-and-system-extensions) | 13 |
| [งานตามกำหนดเวลา](#scheduled-tasks) | 3 |
| [WMI, BITS และทริกเกอร์เหตุการณ์อื่น ๆ](#wmi-bits-and-other-event-triggers) | 11 |
| [Explorer, COM และตัวจัดการไฟล์หรือ URL](#explorer-com-and-file-or-url-handlers) | 15 |
| [แอปพลิเคชันและ add-in ของ Office](#office-applications-and-add-ins) | 7 |
| [ดีบักเกอร์ ตัวจัดการการแครช และ compatibility shim](#debuggers-crash-handlers-and-compatibility-shims) | 13 |
| [การตั้งค่า .NET runtime และโปรไฟเลอร์](#net-runtime-and-profiler-settings) | 7 |
| [การแทนที่การค้นหา DLL และไฟล์ปฏิบัติการ](#dll-and-executable-lookup-overrides) | 18 |
| [บัญชีและการมองเห็นในการลงชื่อเข้าใช้](#accounts-and-sign-in-visibility) | 4 |
| [ส่วนประกอบการยืนยันตัวตนและการป้องกันข้อมูลประจำตัว](#authentication-components-and-credential-protection) | 8 |
| **รวม** | **131** |

## การบูต การลงชื่อเข้าใช้ และการเริ่มทำงาน { #boot-sign-in-and-startup }

การตั้งค่าเหล่านี้กำหนดให้คำสั่งหรือไลบรารีทำงานระหว่างการบูต การลงชื่อเข้าใช้ หรือการเริ่มต้นเดสก์ท็อป ให้ตรวจทานเวอร์ชันของ Windows และผู้ใช้ที่เกี่ยวข้อง เพราะการตั้งค่าอาจล้าสมัย ถูกปิดใช้งาน หรือมีไว้สำหรับเครื่อง kiosk แทนที่จะเป็นของผู้โจมตี

### Active Setup {#active_setup}

**Check ID:** `active_setup` · **ATT&CK ตามแคตตาล็อก:** [T1547.014](https://attack.mitre.org/techniques/T1547/014/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `StubPath` จากการลงทะเบียนคอมโพเนนต์ Active Setup ระดับเครื่องทั้งในมุมมองดั้งเดิมและมุมมอง 32 บิต Windows สามารถใช้คำสั่งเหล่านี้เมื่อเตรียมโปรไฟล์ของผู้ใช้ การตรวจสอบนี้จะข้ามค่าตัวยึดตำแหน่งตามตัวอักษร `U` และ `/UserInstall`

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Active Setup\Installed Components\<component>\StubPath` และโครงสร้าง `Wow6432Node` ที่สอดคล้องกัน

**แนวทางการสืบสวน:** เปรียบเทียบแต่ละคอมโพเนนต์กับผลิตภัณฑ์ที่ติดตั้ง และตรวจสอบอาร์กิวเมนต์ของคำสั่ง การลงทะเบียนระดับเครื่องอาจนำไปสู่การเรียกใช้งานในเซสชันของผู้ใช้ที่กำลังลงชื่อเข้าใช้ ป้าย `SYSTEM` ในเอาต์พุตอธิบายขอบเขตของการกำหนดค่า ไม่ใช่หลักฐานของโทเค็นของโพรเซสที่เกิดขึ้น

### Boot Execute {#boot_execute}

**Check ID:** `boot_execute` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน multi-string `BootExecute` ของ Session Manager โดยเก็บรายการที่ไม่ว่างแต่ละรายการไว้ ซึ่งบันทึกคำสั่งที่กำหนดค่าไว้สำหรับช่วงต้นของการบูต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `BootExecute` |

**แนวทางการสืบสวน:** `autocheck autochk *` เป็นค่าเริ่มต้นที่คุ้นเคย และเครื่องมือจัดการดิสก์อาจเพิ่มรายการที่ถูกต้องได้ ให้สืบสวนคำสั่งที่ถูกเพิ่มและไฟล์ของคำสั่งนั้น โดยไม่สันนิษฐานว่าทุกรายการที่ไม่ใช่ค่าเริ่มต้นเคยทำงานหรือเป็นอันตราย

### Boot Verification Program {#boot_verification_program}

**Check ID:** `boot_verification_program` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `ImagePath` ของ `BootVerificationProgram` ซึ่งเป็นตำแหน่งไฟล์ปฏิบัติการที่เกี่ยวข้องกับการตรวจสอบการบูต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\BootVerificationProgram` | `ImagePath` |

**แนวทางการสืบสวน:** ตรวจสอบว่ากลไกรุ่นเก่านี้ใช้ได้กับเวอร์ชัน Windows และการกำหนดค่าการบูตที่เก็บรวบรวมมาหรือไม่ ระบุไฟล์เป้าหมายและเปรียบเทียบวัตถุประสงค์ของไฟล์กับซอฟต์แวร์จัดการการบูตที่คาดว่าจะมี

### ค่า Load ของ Explorer {#explorer_load}

**Check ID:** `explorer_load` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Load` ที่เกี่ยวข้องกับการกำหนดค่าการเริ่มต้นเมื่อลงชื่อเข้าใช้ Windows จากไฮฟ์ระดับเครื่องและระดับผู้ใช้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows` | `Load` |

**แนวทางการสืบสวน:** สืบสวนคำสั่งที่ไม่คุ้นเคย ปลายทางที่ผู้ใช้เขียนได้ และอาร์กิวเมนต์ที่ดึงหรือถอดรหัสเนื้อหา ยืนยันความสามารถในการใช้งานได้และการเรียกใช้งานแยกกัน ผลลัพธ์ที่ว่างเปล่าไม่ได้ครอบคลุมกลไกการลงชื่อเข้าใช้อื่น ๆ

### Font Drivers {#font_drivers}

**Check ID:** `font_drivers` · **ATT&CK ตามแคตตาล็อก:** [T1547](https://attack.mitre.org/techniques/T1547/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าที่ไม่ว่างในคีย์รีจิสทรีรุ่นเก่า `Font Drivers` ในฐานะการลงทะเบียนไลบรารีที่เป็นไปได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Font Drivers` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ให้ถือว่านี่เป็นพื้นผิวการกำหนดค่ารุ่นเก่าที่ผิดปกติ ตรวจสอบการรองรับตามเวอร์ชัน Windows และไลบรารีที่อ้างอิงก่อนสรุปว่าสามารถโหลดได้ และยืนยันการใช้งานจริงด้วยหลักฐานของโพรเซสหรือการโหลด image

### DLL ส่วนขยาย Group Policy {#gp_extension_dlls}

**Check ID:** `gp_extension_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DllName` จากการลงทะเบียนส่วนขยาย Group Policy ระดับลูกโดยตรงแต่ละรายการภายใต้ Winlogon DLL เหล่านี้ขยายการประมวลผลนโยบาย

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\GPExtensions\<child>` | `DllName` |

**แนวทางการสืบสวน:** ผลิตภัณฑ์การจัดการและความปลอดภัยสามารถติดตั้งส่วนขยายได้ ให้จับคู่ GUID และ DLL ของส่วนขยายกับผลิตภัณฑ์ที่คาดว่าจะมี และตรวจสอบหลักฐานการประมวลผลนโยบายและการโหลด image ก่อนยืนยันว่ามีการเรียกใช้งาน

### สคริปต์ Group Policy {#gp_scripts}

**Check ID:** `gp_scripts` · **ATT&CK ตามแคตตาล็อก:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แยกวิเคราะห์คู่คำสั่ง/พารามิเตอร์จากไฟล์ลงทะเบียน Group Policy ภายในเครื่อง `scripts.ini` และ `psscripts.ini` ที่เก็บรวบรวมมา รวมถึงที่เก็บนโยบายภายในเครื่องรายผู้ใช้ ส่วน startup/shutdown และ logon/logoff จะเก็บคำสั่งและอาร์กิวเมนต์ที่กำหนดค่าไว้ โดยแยกส่วนของเครื่องออกจากส่วนของผู้ใช้

**หลักฐานที่ตรวจสอบ:**

- `Windows\System32\GroupPolicy\Machine\Scripts\scripts.ini` / `psscripts.ini` และไฟล์ `GroupPolicy\User\Scripts` ที่เทียบเท่ากัน
- `Windows\System32\GroupPolicyUsers\<SID>\User\Scripts\scripts.ini` / `psscripts.ini`

**แนวทางการสืบสวน:** ตรวจสอบเฉพาะตำแหน่งลงทะเบียนนโยบายภายในเครื่องที่มีการใช้งานเท่านั้น นี่ไม่ใช่การตรวจสอบ SYSVOL/GPO ที่สมบูรณ์ ไฟล์สคริปต์ที่อยู่ที่อื่นภายใต้ไดเรกทอรีนโยบายยังไม่เพียงพอ ให้เชื่อมโยงการลงทะเบียนกับหลักฐานการประมวลผลนโยบายและการเรียกใช้งาน

### สคริปต์ลงชื่อเข้าใช้ (UserInitMprLogonScript) {#logon_scripts}

**Check ID:** `logon_scripts` · **ATT&CK ตามแคตตาล็อก:** [T1037.001](https://attack.mitre.org/techniques/T1037/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่าสภาพแวดล้อม `UserInitMprLogonScript` ของผู้ใช้ ซึ่งสามารถเลือกสคริปต์สำหรับการลงชื่อเข้าใช้ของผู้ใช้นั้นได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKU\<user>\Environment` | `UserInitMprLogonScript` |

**แนวทางการสืบสวน:** เปรียบเทียบกับสคริปต์ลงชื่อเข้าใช้ที่ได้รับอนุมัติ และตรวจสอบสคริปต์จริง ค่านี้ไม่ได้พิสูจน์ว่ามีการลงชื่อเข้าใช้หรือการเรียกใช้งานตามมา และสคริปต์ที่มาจากโดเมนต้องใช้หลักฐานนโยบายเพิ่มเติม

### Platform Execute {#platform_execute}

**Check ID:** `platform_execute` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `PlatformExecute` ที่ไม่ว่างแต่ละรายการใน Session Manager เพื่อระบุโปรแกรมที่ถูกเลือกสำหรับการเริ่มต้นการบูตที่เกี่ยวข้องกับแพลตฟอร์ม

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `PlatformExecute` |

**แนวทางการสืบสวน:** ขั้นตอนการทำงานของฮาร์ดแวร์หรือการจัดเตรียมระบบ (provisioning) อาจอธิบายรายการได้ ให้พิจารณาว่าเส้นทางการบูตใน Windows build นี้ใช้ค่านี้หรือไม่ และสืบสวน image ที่อ้างอิง แทนที่จะอนุมานว่าเป็นการโจมตีจากชื่อค่า

### DLL ส่วนขยายไคลเอนต์ RDP {#rdp_clx_dll}

**Check ID:** `rdp_clx_dll` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `ClxDllPath` ในการกำหนดค่าผู้ใช้เริ่มต้นของ Terminal Server ในฐานะตำแหน่งส่วนขยายที่เกี่ยวข้องกับ RDP ที่เป็นไปได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\DefaultUserConfiguration` | `ClxDllPath` |

**แนวทางการสืบสวน:** ตรวจสอบเวอร์ชันของ Windows/RDP และส่วนขยายที่ติดตั้ง DLL ที่กำหนดค่าไว้เป็นเบาะแสสำหรับการวิเคราะห์การโหลด image แต่ค่ารีจิสทรีเพียงอย่างเดียวไม่ได้แสดงว่ามีการเชื่อมต่อ RDP หรือการโหลด DLL เกิดขึ้น

### DLL Virtual Channel ของ RDP {#rdp_virtual_channel}

**Check ID:** `rdp_virtual_channel` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าภายใต้การลงทะเบียน `Default\AddIns` ของ Terminal Server Client ระดับเครื่อง ค่าไบนารีจะถูกถอดรหัสเป็น UTF-16LE เมื่อทำได้ เพื่อเก็บพาธส่วนขยายที่ใช้งานได้ไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Terminal Server Client\Default\AddIns\<add-in>`: ค่าทั้งหมด

**แนวทางการสืบสวน:** ไคลเอนต์ RDP และผลิตภัณฑ์เปลี่ยนเส้นทางอุปกรณ์ลงทะเบียน add-in อย่างถูกต้องตามปกติ ให้ตรวจสอบ DLL หรือคำสั่งจริงและเซสชันไคลเอนต์ที่เกี่ยวข้อง ไม่ใช่ทุกค่าภายใต้ add-in จะเป็นพาธไฟล์ปฏิบัติการ

### โปรแกรมเริ่มต้นของ RDP WDS {#rdp_wds_startup}

**Check ID:** `rdp_wds_startup` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `StartupPrograms` จากการกำหนดค่าไดรเวอร์ Windows Station ของ RDP เพื่อระบุโปรแกรมที่กำหนดค่าให้เริ่มเมื่อเริ่มเซสชันระยะไกล

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Terminal Server\Wds\rdpwd` | `StartupPrograms` |

**แนวทางการสืบสวน:** `rdpclip` เป็นรายการที่คุ้นเคย ให้สืบสวนรายการที่ถูกเพิ่มและการกำหนดค่า RDP ของโฮสต์ และใช้เรคคอร์ดของเซสชัน/โพรเซสเพื่อพิจารณาว่าโปรแกรมที่กำหนดค่าไว้เริ่มทำงานจริงหรือไม่

### Registry Run Keys {#run_keys}

**Check ID:** `run_keys` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่า Run, RunOnce, RunEx, RunOnceEx และค่า Run ของนโยบาย Explorer จากไฮฟ์ระดับเครื่อง/ผู้ใช้ รวมถึงคีย์ 32 บิตระดับเครื่องที่ระบุไว้ ส่วน RunEx/RunOnceEx จะถูกไล่ตรวจโดยมีการจำกัดขอบเขตและตรวจสอบวงวน (cycle)

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnce` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunEx` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\RunOnceEx` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Windows\CurrentVersion\Policies\Explorer\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnce` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\CurrentVersion\RunOnceEx` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** โปรแกรมอัปเดตซอฟต์แวร์และแอปพลิเคชันในถาดระบบมักปรากฏที่นี่ ให้เปรียบเทียบไฟล์ปฏิบัติการ อาร์กิวเมนต์ของคำสั่ง และบริบทของผู้ใช้ รายการ RunOnce อาจหายไปหลังจากถูกใช้ สถานะที่เก็บรวบรวมมาไม่ใช่ประวัติที่สมบูรณ์ของการลงชื่อเข้าใช้ก่อนหน้า

### RunServices (รุ่นเก่า) {#run_services}

**Check ID:** `run_services` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าในคีย์ `RunServices` ระดับเครื่องรุ่นเก่า

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServices` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** นี่คือพื้นผิวการกำหนดค่าในอดีต ไม่ได้รับประกันว่า Windows build สมัยใหม่จะเรียกใช้งาน ให้สืบสวนเป้าหมายและความเกี่ยวข้องกับระบบปฏิบัติการ แล้วหาหลักฐานการเรียกใช้งานที่เป็นอิสระ

### RunServicesOnce (รุ่นเก่า) {#run_services_once}

**Check ID:** `run_services_once` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าในคีย์ `RunServicesOnce` ระดับเครื่องรุ่นเก่า

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\RunServicesOnce` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ให้ถือว่าค่าที่ไม่ว่างเป็นเบาะแสด้านการกำหนดค่า ตรวจสอบว่าเวอร์ชัน Windows ต้นทางรองรับกลไกนี้หรือไม่ และซอฟต์แวร์สำหรับการติดตั้งใช้งานหรือความเข้ากันได้อธิบายรายการนี้ได้หรือไม่

### S0 Initial Command {#s0_initial_command}

**Check ID:** `s0_initial_command` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `S0InitialCommand` จาก Session Manager ในฐานะคำสั่งเริ่มต้น Session 0 ที่เป็นไปได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `S0InitialCommand` |

**แนวทางการสืบสวน:** ยืนยันว่าใช้ได้กับ Windows build และสถานะการบูตที่เก็บรวบรวมมา คำสั่งที่กำหนดค่าไว้ไม่ใช่เรคคอร์ดการเรียกใช้งานขณะบูตที่มี timestamp ให้ตรวจสอบไฟล์และเชื่อมโยงกับ telemetry ของการบูต/โพรเซส

### Session Manager Execute {#session_manager_execute}

**Check ID:** `session_manager_execute` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `Execute` ที่ไม่ว่างจาก Session Manager เพื่อแสดงโปรแกรมที่ถูกเลือกสำหรับการเริ่มต้นเซสชัน

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `Execute` |

**แนวทางการสืบสวน:** เปรียบเทียบกับ baseline ของ image ที่คาดว่าจะมี และตรวจสอบไฟล์ปฏิบัติการ native หรือพาธที่ผิดปกติ ยืนยันว่ามีการบูต/เซสชันที่เกี่ยวข้องเกิดขึ้นหลังการเปลี่ยนแปลงก่อนกล่าวอ้างว่ามีการเรียกใช้งาน

### Session Manager SubSystems {#session_manager_subsystems}

**Check ID:** `session_manager_subsystems` · **ATT&CK ตามแคตตาล็อก:** [T1547](https://attack.mitre.org/techniques/T1547/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Windows` ในคีย์ `SubSystems` ของ Session Manager ซึ่งกำหนดค่าคำสั่งของ Windows subsystem

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\SubSystems` | `Windows` |

**แนวทางการสืบสวน:** คำสั่งปกติมีความยาวและขึ้นอยู่กับเวอร์ชัน ให้เปรียบเทียบค่าและอาร์กิวเมนต์ทั้งหมดกับ baseline ที่เชื่อถือได้ของ build เดียวกัน การเปลี่ยนการตั้งค่า subsystem ที่สำคัญยังอาจทำให้การเริ่มระบบล้มเหลวได้ด้วย

### Setup Execute {#setup_execute}

**Check ID:** `setup_execute` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `SetupExecute` ใน Session Manager ซึ่งครอบคลุมคำสั่งที่เกี่ยวข้องกับการติดตั้งที่กำหนดค่าให้ทำงานเมื่อเริ่มระบบ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager` | `SetupExecute` |

**แนวทางการสืบสวน:** กิจกรรมการติดตั้ง การอัปเกรด และการกู้คืนอาจสร้างรายการที่ถูกต้องได้ ให้เปรียบเทียบกับ log การติดตั้งและ image ที่อ้างอิงก่อนจะระบุว่าการกำหนดค่านี้เป็นการคงอยู่

### การเปลี่ยนเส้นทาง Startup ใน Shell Folders {#shell_folders_startup}

**Check ID:** `shell_folders_startup` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** เปรียบเทียบ `Startup` และ `Common Startup` ทั้งใน Shell Folders และ User Shell Folders กับตำแหน่ง Startup เริ่มต้น การเปลี่ยนเส้นทางที่ไม่ใช่ค่าเริ่มต้นจะถูกรายงาน และเนื้อหาของโฟลเดอร์ที่เก็บรวบรวมมาจะถูกตรวจสอบ

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Shell Folders` และ `User Shell Folders`: `Common Startup`
- การตั้งค่า `Software\Microsoft\Windows\CurrentVersion\Explorer` ที่เทียบเท่ากันใน NTUSER.DAT: `Startup` รวมถึงไฟล์ที่เก็บรวบรวมมาในตำแหน่งที่ถูกเปลี่ยนเส้นทาง

**แนวทางการสืบสวน:** โฟลเดอร์ Startup ที่ถูกเปลี่ยนเส้นทางสามารถซ่อนการเริ่มอัตโนมัติได้ แต่การเปลี่ยนเส้นทางก็อาจถูกต้องตามปกติ พาธต้องระบุได้ภายในหลักฐานที่เก็บรวบรวมมา ปลายทางที่ไม่ได้เก็บรวบรวมถือว่าไม่ทราบ ไม่ใช่หลักฐานว่าโฟลเดอร์ว่างเปล่า

### การแทนที่ Shell Launcher {#shell_launcher}

**Check ID:** `shell_launcher` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Shell` ของนโยบายและการแมป Shell สำหรับการบูตของ `system.ini` จาก SOFTWARE ซึ่งเป็นพื้นผิวทางเลือกสำหรับการเลือก shell

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows\System` | `Shell` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\IniFileMapping\system.ini\boot` | `Shell` |

**แนวทางการสืบสวน:** เครื่อง kiosk และอุปกรณ์เฉพาะแอปพลิเคชันอาจตั้งใจแทนที่ Explorer ให้เปรียบเทียบ shell ที่เลือกกับบทบาทของเครื่อง และยืนยันว่าการตั้งค่ามีผลสำหรับเซสชันที่เกี่ยวข้อง

### โฟลเดอร์ Startup {#startup_folder}

**Check ID:** `startup_folder` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** ตรวจสอบไฟล์ที่อยู่โดยตรงในโฟลเดอร์ Startup เริ่มต้นรายผู้ใช้และแบบใช้ร่วมกัน โดยติดตามการเปลี่ยนเส้นทางโฟลเดอร์ Startup แรกที่พบ เป้าหมายของทางลัดจะถูกกู้คืนเมื่อสามารถวิเคราะห์ `.lnk` ที่เก็บรวบรวมมาได้

**หลักฐานที่ตรวจสอบ:**

- `ProgramData\Microsoft\Windows\Start Menu\Programs\Startup` และ `Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup`
- การเปลี่ยนเส้นทาง `Common Startup` / `Startup` ที่เกี่ยวข้องในการตั้งค่า Shell Folders และ User Shell Folders ระดับเครื่อง/ผู้ใช้

**แนวทางการสืบสวน:** ลิงก์และไฟล์ใน Startup สามารถเปิดแอปพลิเคชันที่ถูกต้องเมื่อลงชื่อเข้าใช้ได้ ให้ตรวจสอบอาร์กิวเมนต์ของเป้าหมายรวมถึงชื่อไฟล์ โฟลเดอร์แบบใช้ร่วมกันเป็นตำแหน่งการกำหนดค่าที่ใช้ร่วมกัน ไม่ได้พิสูจน์ว่าโปรแกรมที่ถูกเปิดใช้โทเค็นของ SYSTEM

### Initial Program ของ Terminal Services {#ts_initial_program}

**Check ID:** `ts_initial_program` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `InitialProgram` ของนโยบาย Terminal Services และการกำหนดค่า Run/RunOnce/RunOnceEx สำหรับการติดตั้ง Terminal Server ระดับเครื่อง รวมถึงส่วน RunOnceEx ที่ซ้อนกันภายในขอบเขตที่จำกัด

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Policies\Microsoft\Windows NT\Terminal Services` | `InitialProgram` |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Run` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\Runonce` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Terminal Server\Install\Software\Microsoft\Windows\CurrentVersion\RunOnceEx` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** การติดตั้งใช้งานแอปพลิเคชันระยะไกลอาจตั้งใจใช้การตั้งค่าเหล่านี้ ให้ระบุเซสชันที่ได้รับผลกระทบและติดตามคำสั่งที่เลือกไปยังหลักฐานของโพรเซส การลงทะเบียนเพียงอย่างเดียวไม่ได้แสดงว่ามีการลงชื่อเข้าใช้ระยะไกล

### Winlogon AppSetup {#winlogon_appsetup}

**Check ID:** `winlogon_appsetup` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `AppSetup` จาก Winlogon โดยแยกรายการที่คั่นด้วยจุลภาคออกเป็นผู้ต้องสงสัยแต่ละรายการ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `AppSetup` |

**แนวทางการสืบสวน:** เปรียบเทียบแต่ละคอมโพเนนต์กับซอฟต์แวร์การติดตั้ง/การลงชื่อเข้าใช้ของ Windows ที่คาดว่าจะมี รายการในลิสต์ไม่ใช่การเรียกใช้งานที่สังเกตพบแยกต่างหาก และความเกี่ยวข้องอาจขึ้นอยู่กับเวอร์ชันของ Windows และสถานะการติดตั้งใช้งาน

### Winlogon GinaDLL {#winlogon_gina_dll}

**Check ID:** `winlogon_gina_dll` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าการแทนที่ `GinaDLL` ของ Winlogon รุ่นเก่า GINA เป็นโมเดลส่วนขยายการลงชื่อเข้าใช้ Windows รุ่นเก่า

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `GinaDLL` |

**แนวทางการสืบสวน:** เกี่ยวข้องกับระบบก่อน Vista เป็นหลัก คีย์ที่หลงเหลืออยู่บนระบบรุ่นใหม่ไม่ใช่หลักฐานของส่วนขยายการลงชื่อเข้าใช้ที่มีผลจริง ให้ตรวจสอบเวอร์ชันต้นทางและ DLL ที่อ้างอิง

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://learn.microsoft.com/en-us/windows/win32/secauthn/gina)

### Winlogon MPNotify {#winlogon_mpnotify}

**Check ID:** `winlogon_mpnotify` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าคำสั่ง/ไลบรารี `mpnotify` ของ Winlogon ซึ่งเป็นพื้นผิวส่วนขยายที่เกี่ยวข้องกับการลงชื่อเข้าใช้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `mpnotify` |

**แนวทางการสืบสวน:** เปรียบเทียบเป้าหมายและอาร์กิวเมนต์กับ baseline ของ Windows ที่ตรงกัน ตรวจสอบพฤติกรรมเฉพาะเวอร์ชัน และเชื่อมโยงกับการลงชื่อเข้าใช้ครั้งต่อมาก่อนยืนยันว่าโค้ดได้ทำงาน

### Winlogon Notify Packages {#winlogon_notify_packages}

**Check ID:** `winlogon_notify_packages` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DllName` จากการลงทะเบียนแพ็กเกจ `Notify` ของ Winlogon ระดับลูกโดยตรงแต่ละรายการ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\Notify\<child>` | `DllName` |

**แนวทางการสืบสวน:** notification package เป็นกลไกส่วนขยายรุ่นเก่า ให้พิจารณาว่าเวอร์ชัน Windows ที่เก็บรวบรวมมาโหลดแพ็กเกจเหล่านี้หรือไม่ และแยกซอฟต์แวร์ยืนยันตัวตนที่ได้รับอนุมัติออกจากไลบรารีที่ไม่สามารถอธิบายได้

### Winlogon Shell {#winlogon_shell}

**Check ID:** `winlogon_shell` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Shell` ของ Winlogon ระดับเครื่องและรายผู้ใช้ เพื่อแสดงการเลือก desktop shell

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Shell` |

**แนวทางการสืบสวน:** Explorer เป็นค่าที่พบบ่อย แต่เครื่อง kiosk อาจใช้ shell อื่น ให้สืบสวนคำสั่งที่ถูกเพิ่มหรือพาธที่ผิดปกติ และพิจารณาว่าการกำหนดค่าที่มีผลของผู้ใช้คนใดถูกใช้เมื่อลงชื่อเข้าใช้

### Winlogon System {#winlogon_system}

**Check ID:** `winlogon_system` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `System` ของ Winlogon และแยกรายการที่คั่นด้วยจุลภาคออกเป็นผู้ต้องสงสัยแต่ละรายการ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `System` |

**แนวทางการสืบสวน:** ให้ถือว่านี่เป็นเบาะแสการกำหนดค่าการลงชื่อเข้าใช้ที่ขึ้นกับเวอร์ชัน ตรวจสอบทุกคอมโพเนนต์ที่อ้างอิงและยืนยันด้วยหลักฐานการเรียกใช้งาน แทนที่จะถือว่ารายการเป็นประวัติการทำงาน

### Winlogon Taskman {#winlogon_taskman}

**Check ID:** `winlogon_taskman` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Taskman` ของ Winlogon ระดับเครื่อง/ผู้ใช้ สำหรับคำสั่งจัดการงานทางเลือก

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Taskman` |

**แนวทางการสืบสวน:** พิจารณาว่าพฤติกรรมใดของ Windows จะใช้การตั้งค่านี้ และเป้าหมายเป็นสิ่งที่คาดว่าจะมีหรือไม่ การแทนที่ที่กำหนดค่าไว้ไม่ได้พิสูจน์ว่าผู้ใช้เรียกใช้ฟีเจอร์นั้น

### Winlogon Userinit {#winlogon_userinit}

**Check ID:** `winlogon_userinit` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Userinit` ของ Winlogon ระดับเครื่อง/ผู้ใช้ โดยแยกโปรแกรมที่คั่นด้วยจุลภาคและละเว้นรายการว่างที่อยู่ท้ายสุด

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |
| `HKU\<user>\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `Userinit` |

**แนวทางการสืบสวน:** รายการปกติมักชี้ไปที่ `userinit.exe` ให้ตรวจสอบรายการเพิ่มเติมและพาธที่ถูกแก้ไข ยืนยันผู้ใช้ที่เกี่ยวข้องและการลงชื่อเข้าใช้ในภายหลัง `KeyLastWrite` เป็นของคีย์ทั้งคีย์ ไม่ใช่ของค่านี้โดยเฉพาะ

### Winlogon VMApplet {#winlogon_vmapplet}

**Check ID:** `winlogon_vmapplet` · **ATT&CK ตามแคตตาล็อก:** [T1547.004](https://attack.mitre.org/techniques/T1547/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าคำสั่ง `VMApplet` ของ Winlogon ซึ่งครอบคลุมพื้นผิวการเปิด system applet รุ่นเก่า

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon` | `VMApplet` |

**แนวทางการสืบสวน:** ตรวจสอบความเกี่ยวข้องกับระบบปฏิบัติการและ applet ที่คาดว่าจะมี คำสั่งที่ไม่คาดคิดควรได้รับการสืบสวน แต่ไม่ได้ยืนยันว่ามีการเปิดอินเทอร์เฟซที่เป็นทริกเกอร์

## บริการและส่วนขยายของระบบ { #services-and-system-extensions }

บริการ Windows และคอมโพเนนต์ระบบที่ขยายได้มีวิธีที่ถูกต้องมากมายในการโหลดไฟล์ปฏิบัติการหรือ DLL ให้เปรียบเทียบเป้าหมายที่กำหนดค่าไว้กับผลิตภัณฑ์ที่คาดว่าจะมี บัญชีบริการ การตั้งค่าการเริ่มทำงาน และประวัติการติดตั้งที่มีแหล่งข้อมูลรองรับ

### DiagTrack DLL {#diagtrack_dll}

**Check ID:** `diagtrack_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `ImagePath` ของบริการ DiagTrack แม้ชื่อในแคตตาล็อกจะเน้นเรื่อง DLL แต่การตรวจสอบนี้ตรวจคำสั่ง image ของบริการที่กำหนดค่าไว้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DiagTrack` | `ImagePath` |

**แนวทางการสืบสวน:** เปรียบเทียบกับ baseline ของบริการและการตรวจสอบบริการทั่วไป การกำหนดค่าเดียวกันอาจทำให้เกิดผลการตรวจสอบมากกว่าหนึ่งรายการ จำนวนผู้ต้องสงสัยจึงไม่ใช่จำนวนการโจมตีที่แตกต่างกัน

### DiagTrack Listener DLL {#diagtrack_listener_dll}

**Check ID:** `diagtrack_listener_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `FileName` จากการกำหนดค่า autologger ของ DiagTrack-Listener นี่คือการตั้งค่า ETW autologger ไม่ใช่หลักฐานว่ามีการโหลด DLL

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\WMI\Autologger\DiagTrack-Listener` | `FileName` |

**แนวทางการสืบสวน:** ตรวจสอบค่าดิบและการกำหนดค่าการบันทึกของ Windows แคตตาล็อกระบุว่านี่เป็นเทคนิคด้าน DLL แต่การค้นหานี้ไม่ได้ตรวจสอบว่าเป็น DLL ที่โหลดได้ หรือพิสูจน์การเรียกใช้งานผ่านค่านี้

### Netsh Helper DLL {#netsh_helper}

**Check ID:** `netsh_helper` · **ATT&CK ตามแคตตาล็อก:** [T1546.007](https://attack.mitre.org/techniques/T1546/007/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าที่ไม่ว่างภายใต้คีย์ลงทะเบียน helper ของ `NetSh` ระดับเครื่อง helper สามารถขยาย `netsh.exe` ด้วยคำสั่งที่อิง DLL ได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\NetSh` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ผลิตภัณฑ์ด้านเครือข่ายอาจติดตั้ง helper ให้จับคู่แต่ละค่ากับผลิตภัณฑ์ที่คาดว่าจะมีและตรวจสอบไลบรารีที่ระบุได้ จากนั้นมองหาการเรียกใช้ netsh หรือการโหลด image ที่เกี่ยวข้อง

### Print Monitors {#print_monitors}

**Check ID:** `print_monitors` · **ATT&CK ตามแคตตาล็อก:** [T1547.010](https://attack.mitre.org/techniques/T1547/010/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Driver` ภายใต้การลงทะเบียน Print Monitor ระดับลูกโดยตรงแต่ละรายการใน control set ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Monitors\<child>` | `Driver` |

**แนวทางการสืบสวน:** ซอฟต์แวร์การพิมพ์และ port monitor ขยาย spooler อย่างถูกต้องตามปกติ ให้เปรียบเทียบ DLL และเมทาดาทาของผลิตภัณฑ์กับรายการเครื่องพิมพ์ และยืนยันการโหลดด้วยหลักฐานของ spooler/การโหลด image

### Print Processors {#print_processors}

**Check ID:** `print_processors` · **ATT&CK ตามแคตตาล็อก:** [T1547.012](https://attack.mitre.org/techniques/T1547/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Driver` ภายใต้โครงสร้างการลงทะเบียน print processor ของ x64 และ x86

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows x64\Print Processors\<child>` | `Driver` |
| `HKLM\SYSTEM\<ControlSet>\Control\Print\Environments\Windows NT x86\Print Processors\<child>` | `Driver` |

**แนวทางการสืบสวน:** ชุดซอฟต์แวร์เครื่องพิมพ์ของบุคคลที่สามอาจเพิ่ม processor นอกเหนือจากค่าเริ่มต้นที่คุ้นเคยของ Windows ให้ตรวจทานทั้งสองสถาปัตยกรรม DLL จริง และซอฟต์แวร์เครื่องพิมพ์ที่ติดตั้งก่อนระบุเจตนาร้าย

### ส่วนขยาย Service Control Manager {#scm_extension}

**Check ID:** `scm_extension` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าที่ไม่ว่างใน `ServiceControlManagerExtension` ซึ่งเป็นพื้นผิวส่วนขยายของ service manager

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\ServiceControlManagerExtension` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ตรวจสอบไลบรารีที่ระบุชื่อ และ Windows build นี้ใช้การตั้งค่านี้หรือไม่ ให้เชื่อมโยงกับกิจกรรมการบูต/บริการ การตรวจสอบนี้รวบรวมรายการการลงทะเบียน ไม่ได้พิสูจน์ว่ามีส่วนขยายที่ถูกโหลด

### DNS Server Level Plugin DLL {#server_level_plugin_dll}

**Check ID:** `server_level_plugin_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่า `ServerLevelPluginDll` ของบริการ DNS ใน control set ของ SYSTEM ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\DNS\Parameters` | `ServerLevelPluginDll` |

**แนวทางการสืบสวน:** เกี่ยวข้องกับระบบที่ทำงานในบทบาท DNS Server รวมถึง domain controller บางเครื่อง ให้สืบสวน DLL บัญชีบริการ และหลักฐานการรีสตาร์ต/การโหลดบริการ บทบาทที่ไม่ได้ใช้งานหรือไม่มีอยู่จะเปลี่ยนข้อสรุป

### คำสั่งเมื่อบริการล้มเหลว {#service_failure_command}

**Check ID:** `service_failure_command` · **ATT&CK ตามแคตตาล็อก:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `FailureCommand` ของแต่ละบริการใน control set ที่ใช้งานอยู่ เพื่อแสดงคำสั่งกู้คืนที่กำหนดค่าไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\FailureCommand`

**แนวทางการสืบสวน:** คำสั่งจะทำงานเฉพาะเมื่อ recovery action ของบริการเลือกใช้คำสั่งนั้นและเกิดความล้มเหลวที่เกี่ยวข้อง ให้ตรวจสอบการกำหนดค่าการกู้คืนและเรคคอร์ดความล้มเหลว/โพรเซส ค่านี้เพียงอย่างเดียวไม่ได้ยืนยันว่ามี recovery action ที่ใช้งานอยู่

### SNMP Extension Agent DLL {#snmp_extension_agent}

**Check ID:** `snmp_extension_agent` · **ATT&CK ตามแคตตาล็อก:** [T1574](https://attack.mitre.org/techniques/T1574/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าภายใต้คีย์ลงทะเบียน `ExtensionAgents` ของบริการ SNMP ค่าอาจระบุตำแหน่งรีจิสทรีอื่นแทนที่จะเป็นพาธ DLL โดยตรง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\SNMP\Parameters\ExtensionAgents` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ติดตามการลงทะเบียนทางอ้อมด้วยตนเองไปจนถึงไลบรารี และตรวจสอบว่ามีการติดตั้งและใช้งาน SNMP สตริงที่ระบุไม่ได้ในเอาต์พุตนี้ไม่ใช่หลักฐานว่า DLL หายไปหรือเป็นอันตราย

### Time Providers {#time_providers}

**Check ID:** `time_providers` · **ATT&CK ตามแคตตาล็อก:** [T1547.003](https://attack.mitre.org/techniques/T1547/003/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DllName` จากการลงทะเบียน `TimeProviders` ของ Windows Time ระดับลูกโดยตรง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\W32Time\TimeProviders\<child>` | `DllName` |

**แนวทางการสืบสวน:** ตรวจทานการตั้งค่าการเปิดใช้งาน provider และสถานะของบริการ Windows Time แยกกัน การตรวจสอบพาธ DLL ไม่ได้กำหนดว่า provider ต้องเปิดใช้งานอยู่ ผลิตภัณฑ์ซิงโครไนซ์เวลาอาจมีตัวแทนที่ถูกต้องตามปกติ

### บริการ Windows (ServiceDll) {#windows_service_dll}

**Check ID:** `windows_service_dll` · **ATT&CK ตามแคตตาล็อก:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Parameters\ServiceDll` ภายใต้บริการต่าง ๆ ใน control set ของ SYSTEM ที่ใช้งานอยู่ และแนบหลักฐาน timestamp เฉพาะบริการเมื่อมีข้อมูล

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Parameters\ServiceDll`

**แนวทางการสืบสวน:** โฮสต์ที่ใช้ร่วมกัน เช่น `svchost.exe` ไม่ใช่ DLL ที่เป็น payload ให้ตรวจสอบไลบรารี บัญชีบริการ สถานะการเริ่มทำงาน และหลักฐานการติดตั้งบริการ/การโหลด image ที่ตรงกัน การลงทะเบียนไม่ได้พิสูจน์ว่าบริการได้ทำงาน

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)

### บริการ Windows (ImagePath) {#windows_service_image_path}

**Check ID:** `windows_service_image_path` · **ATT&CK ตามแคตตาล็อก:** [T1543.003](https://attack.mitre.org/techniques/T1543/003/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง `ImagePath` ของบริการใน control set ที่ใช้งานอยู่ ไดรเวอร์ชนิด 1, 2 หรือ 8 ที่ไม่มี ImagePath จะคงเป้าหมายโดยนัย `System32\drivers\<service>.sys` ไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>`: `ImagePath`, `Type` และเมทาดาทาของบริการที่เกี่ยวข้อง

**แนวทางการสืบสวน:** ตรวจทานบัญชีบริการและการตั้งค่าการเริ่มทำงานแยกจากป้ายขอบเขตระดับเครื่องในเอาต์พุต การลงทะเบียนที่ถูกปิดใช้งานหรือเริ่มเมื่อมีการร้องขอก็ยังถือเป็นรายการในคลัง สำหรับ timestamp ให้ใช้หลักฐานจากชื่อบริการเป็นหลัก แทนพาธไฟล์ปฏิบัติการที่ใช้ร่วมกันซึ่งคลุมเครือ

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://docs.microsoft.com/en-us/windows/win32/services/service-programs)

### Startup DLL ของบริการ Windows Update {#wu_service_startup_dll}

**Check ID:** `wu_service_startup_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Parameters\ServiceDll` ของบริการ Windows Update จาก control set ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\wuauserv\Parameters` | `ServiceDll` |

**แนวทางการสืบสวน:** เปรียบเทียบไลบรารีกับ Windows build เดียวกันและสถานะ servicing ที่ติดตั้ง การตรวจสอบนี้ซ้อนทับกับรายการ ServiceDll ทั่วไป และไม่ได้แสดงถึงการบุกรุกครั้งที่สองที่แยกจากกัน

## งานตามกำหนดเวลา { #scheduled-tasks }

Task Scheduler จัดเก็บการกำหนดค่าทั้งในไฟล์ XML และไฮฟ์ SOFTWARE ให้อ่านแหล่งข้อมูลเหล่านั้นร่วมกัน งานอาจถูกลงทะเบียนโดยไม่เคยทำงาน และการเก็บรวบรวมที่ไม่ครบถ้วนอาจดูคล้ายงานที่ถูกซ่อนหรืองาน ghost

### งานตามกำหนดเวลาแบบ Ghost {#ghost_task}

**Check ID:** `ghost_task` · **ATT&CK ตามแคตตาล็อก:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**สิ่งที่ Juntei ตรวจสอบ:** เปรียบเทียบการลงทะเบียน `Tree` และ `Tasks` ของ TaskCache ใน SOFTWARE และเมื่อมีไฟล์งานในข้อมูลที่เก็บรวบรวมมา จะค้นหาไฟล์ XML ของงานที่สอดคล้องกัน

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree` และ `TaskCache\Tasks`
- ไฟล์ `Windows\System32\Tasks\<task path>` ที่เก็บรวบรวมมาสำหรับการเปรียบเทียบกับระบบไฟล์

**แนวทางการสืบสวน:** การที่ไม่มีส่วนที่คู่กันอาจสะท้อนถึงงานที่ถูกลบ/ค้างอยู่ การลงทะเบียนในตัว หรือการเก็บรวบรวมที่ไม่ครบถ้วน หากไม่มี XML ที่เก็บรวบรวมมา การเปรียบเทียบกับระบบไฟล์จะทำไม่ได้ ให้ตรวจสอบ coverage และความไม่สอดคล้องที่แน่ชัดก่อนเรียกงานว่าถูกซ่อนหรือเป็นอันตราย

### งานตามกำหนดเวลาที่ถูกซ่อน {#hidden_scheduled_task}

**Check ID:** `hidden_scheduled_task` · **ATT&CK ตามแคตตาล็อก:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**สิ่งที่ Juntei ตรวจสอบ:** ตรวจสอบการลงทะเบียน Tree ของ TaskCache ว่าไม่มีค่า `SD` หรือมี security descriptor แบบไบนารีที่ว่างเปล่า ซึ่งเป็นเงื่อนไขที่เกี่ยวข้องกับการซ่อนงานจากการแจกแจงตามปกติ

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Schedule\TaskCache\Tree\<task>`: การมีอยู่/ข้อมูลของ `SD` พร้อมเมทาดาทาของส่วนที่คู่กันใน TaskCache

**แนวทางการสืบสวน:** SD ที่ขาดหาย/ว่างเปล่าเป็นความผิดปกติของการกำหนดค่าเฉพาะอย่าง ไม่ใช่หลักฐานว่างานได้ทำงานหรือเป็นอันตราย การตั้งค่า `Hidden` ใน XML และปัญหา ACL ทั่วไปเป็นเงื่อนไขที่ต่างกัน และไม่ใช่เงื่อนไขที่ใช้ในที่นี้

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://www.microsoft.com/en-us/security/blog/2022/04/12/tarrask-malware-uses-scheduled-task-defense-evasion/)

### งานตามกำหนดเวลา (ไฟล์ XML) {#scheduled_task_files}

**Check ID:** `scheduled_task_files` · **ATT&CK ตามแคตตาล็อก:** [T1053.005](https://attack.mitre.org/techniques/T1053/005/)

**สิ่งที่ Juntei ตรวจสอบ:** แยกวิเคราะห์ XML ของงานที่เก็บรวบรวมมาเพื่อหาคำสั่ง/อาร์กิวเมนต์ของ Exec และ class ID/ข้อมูลของ COM handler โดยเลือก principal ตาม `Actions Context`/`Principal id` เก็บตัวตนของผู้ใช้และกลุ่มแยกจากกัน และรายงานเป็น SYSTEM, USER หรือ UNKNOWN action แบบ COM ที่ระบุไม่ได้จะยังคงแสดงอยู่

**หลักฐานที่ตรวจสอบ:**

- ไฟล์ XML `Windows\System32\Tasks\<task path>` รวมถึง Exec, ComHandler และฟิลด์ principal ที่มี
- `UsrClass.dat` และ `Software\Classes` ใน NTUSER ของผู้ใช้ที่ตรงกันที่เก็บรวบรวมมา รวมถึง `Classes\CLSID` ใน SOFTWARE ระดับเครื่อง ทั้งในมุมมองดั้งเดิมและมุมมอง 32 บิต
- `ProfileList` ใน SOFTWARE ที่เก็บรวบรวมมาสำหรับ principal ที่เป็น SID สำหรับ principal ที่เป็นชื่อ จะเลือกโฟลเดอร์ `user.DOMAIN` ที่ตรงกันแบบไม่ซ้ำก่อน จากนั้นจึงเป็น suffix ของโฮสต์ภายในเครื่องที่เก็บรวบรวมมา และสุดท้ายคือโฟลเดอร์ `user` ธรรมดา การจับคู่จะระบุอย่างชัดเจนว่าเป็นการอนุมานแบบ heuristic ความคลุมเครือที่แท้จริงจะคงสถานะระบุไม่ได้และบันทึกไว้ในผลการตรวจพบและ coverage ไม่จำเป็นต้องใช้คีย์รีจิสทรีแบบ volatile
- `CodeBase` แบบ managed (รวมถึงซับคีย์เวอร์ชัน) ความเชื่อมโยงกับ AppID และ `ImagePath` ของบริการใน SYSTEM สำหรับ `LocalService` ของ AppID

ผู้ต้องสงสัยที่เป็น COM server จะมีแหล่งที่มาในรีจิสทรีอยู่ใน `Description` ความเชื่อมโยงกับ AppID
ที่มีเพียงชื่อไฟล์ปฏิบัติการจะคงสถานะระบุไม่ได้ แทนที่จะถูกกำหนดไดเรกทอรีโดยการเดา URL แบบ
`file:` ภายในเครื่องจะถูกถอดรหัส ส่วน URL ระยะไกลจะถูกเก็บไว้โดยไม่เข้าถึงเครือข่าย การแทนที่คลาสของผู้ใช้
มีขอบเขตเฉพาะโปรไฟล์นั้น และ CLSID ของผู้ใช้ที่ไม่สมบูรณ์จะไม่ถูกส่งต่อไปยังการลงทะเบียนระดับเครื่อง
ที่ถูกบดบังอยู่ งานที่มีเพียงกลุ่มจะคงผู้ต้องสงสัยระดับเครื่องไว้ โดยระบุข้อจำกัดเรื่องผู้ใช้รายบุคคล
ไว้ในคำอธิบาย โดยไม่เพิ่มประกาศตามปกติลงในสรุปข้อผิดพลาดของ coverage

**แนวทางการสืบสวน:** ตรวจสอบทริกเกอร์ สถานะการเปิดใช้งาน principal และประวัติการทำงานใน XML ต้นทางและ event log งานที่ถูกสร้างอาจไม่เคยทำงานเลย COM handler ที่ระบุไม่ได้ยังคงเป็นผู้ต้องสงสัย และต้องไม่สันนิษฐานว่าข้อมูล principal ที่ขาดหาย/มีเพียงกลุ่มหมายถึง SYSTEM

## WMI, BITS และทริกเกอร์เหตุการณ์อื่น ๆ { #wmi-bits-and-other-event-triggers }

Windows Management Instrumentation (WMI) รองรับการจัดการระบบ Background Intelligent Transfer Service (BITS) รองรับการถ่ายโอนข้อมูลเบื้องหลัง ทั้งสองอย่างสามารถเก็บคำสั่งที่ถูกทริกเกอร์ด้วยเหตุการณ์ไว้ได้ กลไกบางอย่างรอเหตุการณ์การจัดการ การแจ้งเตือนการถ่ายโอน การเปิดแอปพลิเคชัน หรือการกระทำของผู้ใช้ ทริกเกอร์ที่จัดเก็บไว้เป็นหลักฐานการกำหนดค่าที่มีประโยชน์ แต่การมีอยู่ของทริกเกอร์ไม่ได้ยืนยันว่าทริกเกอร์นั้นทำงานแล้ว

### Backdoor ผ่าน Accessibility Features {#accessibility_tools}

**Check ID:** `accessibility_tools` · **ATT&CK ตามแคตตาล็อก:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**สิ่งที่ Juntei ตรวจสอบ:** เปรียบเทียบไบนารี accessibility ใน System32 เจ็ดไฟล์ที่เก็บรวบรวมมา โดยใช้ PE OriginalFilename และเมทาดาทาของใบรับรอง/catalog หลังจากกำหนดให้ต้องมีเมทาดาทา catalog ของ Microsoft ที่ใช้งานได้จากไบนารีอ้างอิงที่เก็บรวบรวมมา

**หลักฐานที่ตรวจสอบ:**

- `Windows\System32\sethc.exe`, `osk.exe`, `narrator.exe`, `magnify.exe`, `utilman.exe`, `atbroker.exe` และ `DisplaySwitch.exe`
- เมทาดาทาอ้างอิงของ `Windows\System32\winlogon.exe`, `cmd.exe` หรือ `notepad.exe` ที่เก็บรวบรวมมา และ catalog ที่เกี่ยวข้องที่เก็บรวบรวมมา

**แนวทางการสืบสวน:** เมทาดาทาอาจบ่งชี้ถึงไฟล์ทดแทนที่ถูกเปลี่ยนชื่อ แต่ Juntei ไม่ได้ตรวจสอบความถูกต้องของลายเซ็นหรือความน่าเชื่อถือของใบรับรอง การเก็บรวบรวมที่มีข้อมูลน้อยโดยไม่มี catalog ที่จำเป็นจะทำให้ coverage ของการเปรียบเทียบไม่สมบูรณ์ แทนที่จะถือว่าไฟล์ทุกไฟล์ที่ดูเหมือนไม่ได้ลงนามเป็น backdoor

### Accessibility Features {#assistive_technology}

**Check ID:** `assistive_technology` · **ATT&CK ตามแคตตาล็อก:** [T1546.008](https://attack.mitre.org/techniques/T1546/008/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง `StartExe` พร้อม `StartParams` ของ accessibility ที่ลงทะเบียนไว้ในมุมมอง SOFTWARE ดั้งเดิม/32 บิต และรายการ `Configuration` ของ accessibility ที่เปิดใช้งานจากไฮฟ์ผู้ใช้และไฮฟ์ DEFAULT ตัวยึดตำแหน่ง StartExe ที่เป็นตัวเลข/ไม่ใช่สตริงจะถูกข้าม

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Accessibility\ATs\<feature>`: `StartExe` และ `StartParams` รวมถึงมุมมอง `Wow6432Node` ระดับเครื่อง
- `HKU\<user>\Software\Microsoft\Windows NT\CurrentVersion\Accessibility\Configuration` ใน NTUSER.DAT และการตั้งค่าที่เทียบเท่ากันในไฮฟ์ DEFAULT

**แนวทางการสืบสวน:** ซอฟต์แวร์ accessibility เป็นซอฟต์แวร์ที่ถูกต้องตามปกติ ให้แยกชื่อฟีเจอร์ที่เปิดใช้งานออกจากการลงทะเบียนไฟล์ปฏิบัติการจริง จากนั้นตรวจสอบเป้าหมายและหลักฐานการเรียกใช้งานเมื่อลงชื่อเข้าใช้/บนเดสก์ท็อป การตรวจสอบนี้แยกจากการแทนที่ไฟล์ปฏิบัติการ accessibility เอง

**รายละเอียดเพิ่มเติม:** [ข้อมูลอ้างอิงเทคนิค (hexacorn.com)](https://www.hexacorn.com/blog/2016/07/22/beyond-good-ol-run-key-part-42/)

### คำสั่งแจ้งเตือนของ BITS Job {#bits_notify_command}

**Check ID:** `bits_notify_command` · **ATT&CK ตามแคตตาล็อก:** [T1197](https://attack.mitre.org/techniques/T1197/)

**สิ่งที่ Juntei ตรวจสอบ:** กู้คืนเรคคอร์ด BITS job แบบ counted-string จาก `qmgr.db`, `qmgr0.dat` หรือ `qmgr1.dat` ที่เก็บรวบรวมมา โดยเก็บโปรแกรมแจ้งเตือนที่ไม่ว่าง พารามิเตอร์ job ID และ SID ของเจ้าของไว้

**หลักฐานที่ตรวจสอบ:**

- `ProgramData\Microsoft\Network\Downloader\qmgr.db`, `qmgr0.dat` และ `qmgr1.dat`

**แนวทางการสืบสวน:** เรคคอร์ดเหล่านี้ได้มาจากการ carve สถานะ job ที่จัดสรร/ปัจจุบันจะไม่ได้รับการตรวจสอบ และ job ที่ค้างอยู่/ถูกลบอาจปรากฏขึ้น เวลาเขียนใน MFT ของฐานข้อมูลเป็นหลักฐานระดับคอนเทนเนอร์ที่อ่อน ไม่ใช่เวลาที่การแจ้งเตือนถูกเรียกใช้งาน SID ของเจ้าของช่วยแยกแยะ SYSTEM, local service, network service และผู้ใช้

### Command Processor AutoRun {#cmd_autorun}

**Check ID:** `cmd_autorun` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง `AutoRun` ของ Command Processor ระดับเครื่อง/ผู้ใช้ ซึ่งเป็นการตั้งค่าที่อาจส่งผลต่อการเริ่มทำงานของ command prompt

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |
| `HKU\<user>\SOFTWARE\Microsoft\Command Processor` | `AutoRun` |

**แนวทางการสืบสวน:** เครื่องมือพัฒนาและการปรับแต่ง shell อาจอธิบายค่านี้ได้ `cmd.exe /d` ข้าม AutoRun ดังนั้นการกำหนดค่าเพียงอย่างเดียวไม่ได้พิสูจน์ว่าคำสั่งทำงานทุกครั้งที่ cmd.exe ปรากฏ ให้ตรวจสอบเนื้อหาของคำสั่งและอาร์กิวเมนต์ของโพรเซสที่เป็นทริกเกอร์

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/cmd)

### การคงอยู่ผ่าน ErrorHandler.cmd {#error_handler_cmd}

**Check ID:** `error_handler_cmd` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** รายงานการมีอยู่ของสคริปต์ `Windows\System32\ErrorHandler.cmd` ที่เก็บรวบรวมมา ในฐานะผู้ต้องสงสัยการคงอยู่ผ่าน error handler

**หลักฐานที่ตรวจสอบ:**

- `Windows\System32\ErrorHandler.cmd`

**แนวทางการสืบสวน:** การมีอยู่เพียงอย่างเดียวไม่ได้พิสูจน์ว่าเส้นทางการกู้คืน/ข้อผิดพลาดของ Windows ที่เกี่ยวข้องได้เรียกใช้สคริปต์นี้ ให้อ่านสคริปต์ ตรวจสอบที่มา และเชื่อมโยงกับหลักฐานการติดตั้ง/ข้อผิดพลาด/โพรเซส

### Power Automate Desktop Flows {#power_automate}

**Check ID:** `power_automate` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาไดเรกทอรี flow/สคริปต์ที่เก็บรวบรวมมาภายใต้ที่เก็บ `flows` และ `scripts` ของ Power Automate Desktop ของผู้ใช้แต่ละคน โดยรายงานแต่ละ flow ที่พบเพียงครั้งเดียว

**หลักฐานที่ตรวจสอบ:**

- `Users\<user>\AppData\Local\Microsoft\Power Automate Desktop\flows\<flow>\...` และ `scripts\<flow>\...`

**แนวทางการสืบสวน:** การตรวจสอบนี้ระบุระบบอัตโนมัติที่จัดเก็บไว้ ไม่ใช่ทริกเกอร์ตามกำหนดเวลาทุกรายการหรือ flow ที่ถูกเรียกใช้แล้ว ให้ตรวจสอบเนื้อหาของ flow และการกำหนดค่าการเปิดจากภายนอก ก่อนถือว่าเป็นการคงอยู่แบบอัตโนมัติหรือเป็นอันตราย

### PowerShell Profile {#powershell_profiles}

**Check ID:** `powershell_profiles` · **ATT&CK ตามแคตตาล็อก:** [T1546.013](https://attack.mitre.org/techniques/T1546/013/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาไฟล์รูปแบบ profile.ps1 ในไดเรกทอรีการติดตั้ง/โปรไฟล์ของ Windows PowerShell และ PowerShell ที่รู้จัก รวมถึง Documents ของผู้ใช้ OneDrive Documents ที่พบ และการเปลี่ยนเส้นทาง Documents ที่เก็บรวบรวมมา

**หลักฐานที่ตรวจสอบ:**

- ไฟล์ที่ลงท้ายด้วย `profile.ps1` ที่อยู่โดยตรงใน `Windows\System32\WindowsPowerShell\v1.0` และ `Windows\SysWOW64\WindowsPowerShell\v1.0`
- `Documents\WindowsPowerShell` และ `Documents\PowerShell` รายผู้ใช้ รวมถึงการเปลี่ยนเส้นทาง Documents ที่เก็บรวบรวมมาและโฟลเดอร์ OneDrive Documents ที่พบ
- ไฟล์ที่ลงท้ายด้วย `profile.ps1` ที่อยู่โดยตรงในไดเรกทอรีเวอร์ชันภายใต้ `Program Files\PowerShell` หรือ `Program Files (x86)\PowerShell`

**แนวทางการสืบสวน:** โปรไฟล์อาจมีการปรับแต่งที่ไม่เป็นอันตราย ให้อ่านสคริปต์และติดตาม payload ที่ถูกเรียกใช้ การเปิด PowerShell ด้วย `-NoProfile` จะข้ามโปรไฟล์เหล่านี้ การมีอยู่ของไฟล์เพียงอย่างเดียวไม่ได้พิสูจน์ว่าโปรไฟล์ถูกเรียกใช้หรือมีเนื้อหาที่เป็นอันตราย

### การ Hijack Screensaver {#screensaver}

**Check ID:** `screensaver` · **ATT&CK ตามแคตตาล็อก:** [T1546.002](https://attack.mitre.org/techniques/T1546/002/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `SCRNSAVE.EXE` ใน Desktop ของผู้ใช้แต่ละคน เพื่อระบุ image ของ screensaver ที่กำหนดค่าไว้

**หลักฐานที่ตรวจสอบ:**

- `HKU\<user>\Control Panel\Desktop\SCRNSAVE.EXE` ใน NTUSER.DAT

**แนวทางการสืบสวน:** เปรียบเทียบกับ screensaver ที่ได้รับอนุมัติและตรวจสอบเป้าหมาย การตั้งค่าเดสก์ท็อปอื่น ๆ สถานะเซสชัน และระยะหมดเวลาเป็นตัวกำหนดว่าจะทริกเกอร์ได้หรือไม่ พาธที่บันทึกไว้ไม่ได้พิสูจน์ว่า screensaver เริ่มทำงาน

### คำสั่งของ Telemetry Controller {#telemetry_controller}

**Check ID:** `telemetry_controller` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Binary`, `Command` และ `MaintenanceCommand` จาก TelemetryController และการลงทะเบียนระดับลูกโดยตรง

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\TelemetryController` และลูกโดยตรง: `Binary`, `Command`, `MaintenanceCommand`

**แนวทางการสืบสวน:** เปรียบเทียบไฟล์ปฏิบัติการและอาร์กิวเมนต์ที่เลือกกับ Windows build เดียวกันและซอฟต์แวร์บำรุงรักษา คำสั่งในรีจิสทรีไม่ได้แสดงว่าทริกเกอร์การบำรุงรักษาได้ทำงาน

### โปรไฟล์กำหนดเองของ Windows Terminal {#windows_terminal}

**Check ID:** `windows_terminal` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** แยกวิเคราะห์การตั้งค่า JSON/JSONC ของ Windows Terminal แบบ packaged, preview และ unpackaged และรายงานการตั้งค่า `commandline` ของโปรไฟล์/ค่าเริ่มต้นที่ไม่ว่าง

**หลักฐานที่ตรวจสอบ:**

- `Users\<user>\AppData\Local\Packages\Microsoft.WindowsTerminal_8wekyb3d8bbwe\LocalState\settings.json` และไฟล์ที่เทียบเท่ากันของ WindowsTerminalPreview
- `Users\<user>\AppData\Local\Microsoft\Windows Terminal\settings.json`: อาร์เรย์โปรไฟล์รุ่นเก่า `profiles.list` และฟิลด์ commandline ของ `profiles.defaults`

**แนวทางการสืบสวน:** shell ทั่วไปย่อมปรากฏที่นี่ตามปกติ ให้พิจารณาโปรไฟล์ที่เลือก/ค่าเริ่มต้น และตรวจสอบคำสั่งหรืออาร์กิวเมนต์การเปิดที่ผิดปกติ การบันทึกโปรไฟล์เพียงอย่างเดียวไม่ได้ทำให้โปรไฟล์ถูกเปิด

### WMI Event Subscription {#wmi_event_subscription}

**Check ID:** `wmi_event_subscription` · **ATT&CK ตามแคตตาล็อก:** [T1546.003](https://attack.mitre.org/techniques/T1546/003/)

**สิ่งที่ Juntei ตรวจสอบ:** carve อินสแตนซ์ event consumer ที่รู้จักและการอ้างอิง binding จาก filter ไปยัง consumer จาก repository WMI OBJECTS.DATA ที่เก็บรวบรวมมา คลาสที่รู้จักได้แก่ consumer แบบ command-line, active-script, log-file, event-log และ SMTP

**หลักฐานที่ตรวจสอบ:**

- `Windows\System32\wbem\Repository\OBJECTS.DATA` และ `Repository\FS\OBJECTS.DATA`

**แนวทางการสืบสวน:** ผลที่พบใน repository ดิบอาจค้างอยู่ ไม่ได้ผูก (unbound) หรือเป็นข้อมูลในอดีต เอาต์พุตจะคงข้อสังเกตนั้นไว้ การ carve ไม่ได้สร้างกราฟ subscription ที่ใช้งานอยู่ที่สมบูรณ์ขึ้นใหม่ หรือพิสูจน์ว่าทริกเกอร์ทำงานแล้ว เวลาเขียนไฟล์ของ repository ไม่ใช่เวลาที่สร้าง/เรียกใช้ subscription

## Explorer, COM และตัวจัดการไฟล์หรือ URL { #explorer-com-and-file-or-url-handlers }

Component Object Model (COM) ช่วยให้แอปพลิเคชันเปิดใช้งานคอมโพเนนต์ซอฟต์แวร์ที่ลงทะเบียนไว้ได้ Windows shell ใช้ตัวจัดการ (handler) ที่ลงทะเบียนไว้เพื่อเปิดไฟล์ URL โฟลเดอร์ และออบเจกต์พิเศษ ตัวระบุ COM (CLSID) ใช้ระบุคอมโพเนนต์ ส่วน ProgID ให้ชื่อที่อ่านได้แก่คอมโพเนนต์บางตัว ให้ติดตามการลงทะเบียนแต่ละรายการไปจนถึงคำสั่งหรือ DLL ที่ถูกเลือก

### การ Hijack App Paths {#app_paths}

**Check ID:** `app_paths` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่าเริ่มต้นภายใต้การลงทะเบียน `App Paths` ระดับเครื่อง ซึ่งสามารถเปลี่ยนเส้นทางการค้นหาแอปพลิเคชันสำหรับการเปิดผ่าน shell ได้

**หลักฐานที่ตรวจสอบ:**

- ค่าเริ่มต้นของ `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\App Paths\<application>`

**แนวทางการสืบสวน:** การลงทะเบียน App Paths พบได้ทั่วไปสำหรับแอปพลิเคชันที่ติดตั้ง และไม่ใช่การเขียนทับการเปิดโพรเซสทุกครั้งโดยทั่วไป ให้เปรียบเทียบชื่อและ image ที่ลงทะเบียนกับซอฟต์แวร์ที่คาดว่าจะมี จากนั้นตรวจสอบหลักฐานของ shell/โพรเซสที่เกี่ยวข้อง

### การ Hijack COM Object {#com_treat_as}

**Check ID:** `com_treat_as` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาการเปลี่ยนเส้นทาง `TreatAs` ของ COM และการลงทะเบียน InprocServer32/LocalServer32 รายผู้ใช้ที่บดบัง CLSID ระดับเครื่อง ในที่เก็บคลาสแบบดั้งเดิม/32 บิตที่รองรับ เป้าหมายของ TreatAs จะถูกระบุโดยใช้คลาสของผู้ใช้คนเดียวกันและคลาสระดับเครื่อง

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Classes\CLSID\<CLSID>` และมุมมอง `Classes\Wow6432Node` ระดับเครื่อง
- `Software\Classes\CLSID` รายผู้ใช้ใน NTUSER.DAT และ `CLSID` ใน UsrClass.dat รวมถึงมุมมอง `Wow6432Node` ที่มีการใช้งาน: ค่าเริ่มต้นของ `TreatAs`, `InprocServer32` และ `LocalServer32`

**แนวทางการสืบสวน:** การแทนที่รายผู้ใช้อาจไม่ต้องใช้สิทธิ์ผู้ดูแลระบบ แต่ซอฟต์แวร์ที่ถูกต้องก็ลงทะเบียนสิ่งเหล่านี้ได้เช่นกัน ให้ติดตามคลาสที่ถูกเลือกและ DLL/ไฟล์ปฏิบัติการ ประเมินสถาปัตยกรรมและบริบทการเปิดใช้งาน และยืนยันการเปิดใช้งาน COM จริงด้วยหลักฐานที่เป็นอิสระ

### การ Hijack Disk Cleanup Handler {#disk_cleanup_handler}

**Check ID:** `disk_cleanup_handler` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน CLSID ของ handler ใน VolumeCaches ของ Disk Cleanup และติดตามการลงทะเบียน COM InprocServer32 ระดับเครื่องที่ระบุได้ไปจนถึงไลบรารี

**หลักฐานที่ตรวจสอบ:**

- CLSID เริ่มต้นของ `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\VolumeCaches\<handler>` และ `Classes\CLSID\<CLSID>\InprocServer32`

**แนวทางการสืบสวน:** handler ต้องถูกเรียกโดยขั้นตอนการทำความสะอาดที่เกี่ยวข้องจึงจะทำงาน handler ที่ระบุไม่ได้จะไม่ถูกส่งออกโดยการตรวจสอบนี้ ดังนั้นหากข้อมูลที่เก็บรวบรวมไม่สมบูรณ์ ให้ใช้พาร์เซอร์รีจิสทรีเพื่อตรวจสอบการลงทะเบียนดิบ

### การแทนที่ AppKey ของ Explorer {#explorer_app_key}

**Check ID:** `explorer_app_key` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `ShellExecute` และ `Association` ภายใต้ลูกโดยตรงของ `AppKey` ของ Explorer ซึ่งครอบคลุมคำสั่ง/การกระทำที่ถูกเลือกโดย application key

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\AppKey\<key>`: `ShellExecute` และ `Association`

**แนวทางการสืบสวน:** ยูทิลิตีคีย์บอร์ดและเครื่องมือของ OEM สามารถกำหนดค่าเหล่านี้ได้ ชื่อ association อาจต้องใช้การเชื่อมโยงรีจิสทรีเพิ่มเติม แทนที่จะเป็นพาธไฟล์ ให้ตรวจสอบการกระทำของผู้ใช้ที่จะทริกเกอร์ค่านี้

### Browser Helper Objects {#explorer_bho}

**Check ID:** `explorer_bho` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการลงทะเบียน CLSID ของ Browser Helper Object และระบุไลบรารี InprocServer32 ระดับเครื่องเมื่อมีข้อมูล class ID ที่ระบุไม่ได้จะยังคงแสดงอยู่

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\Browser Helper Objects\<CLSID>` และค่าเริ่มต้นของ `Classes\CLSID\<CLSID>\InprocServer32` ระดับเครื่อง

**แนวทางการสืบสวน:** การตรวจสอบนี้ผูกกับพฤติกรรมของเบราว์เซอร์/Explorer ที่เข้ากันได้ และเกี่ยวข้องเป็นพิเศษกับสภาพแวดล้อมรุ่นเก่า ส่วนขยายเบราว์เซอร์/ความปลอดภัยที่ได้รับอนุมัติอาจปรากฏได้ BHO ที่ลงทะเบียนไว้ไม่ใช่หลักฐานว่าเบราว์เซอร์ปัจจุบันได้โหลด BHO นั้น

### การ Hijack CLSID ของ Explorer Desktop {#explorer_clsid_hijack}

**Check ID:** `explorer_clsid_hijack` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง shell-open ที่กำหนดค่าไว้บน CLSID ของ Explorer namespace พิเศษสามรายการที่ระบุไว้ จากที่เก็บคลาสระดับเครื่อง/ผู้ใช้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{52205fd8-5dfb-447d-801a-d0b52f2e83e1}\shell\opennewwindow\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{20D04FE0-3AEA-1069-A2D8-08002B30309D}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{450D8FBA-AD25-11D0-98A8-0800361B1103}\shell\open\command` | `(Default)` |

**แนวทางการสืบสวน:** เปรียบเทียบตัวจัดการคำสั่งของ Desktop, My Computer และ My Documents กับ baseline ที่ตรงกัน การตรวจสอบนี้ตรวจเฉพาะ CLSID และพาธที่เลือกไว้ ไม่ใช่การ hijack Explorer namespace ทุกรูปแบบที่เป็นไปได้

### ตัวจัดการเมนูบริบทของ Explorer {#explorer_context_menu}

**Check ID:** `explorer_context_menu` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงตัวจัดการเมนูบริบทสำหรับ Directory, Directory Background, Folder, Drive, AllFilesystemObjects และ `*` โดยติดตาม class ID ไปยังการลงทะเบียน DLL ที่เก็บรวบรวมมาเมื่อทำได้

**หลักฐานที่ตรวจสอบ:**

- ที่เก็บคลาส `HKLM\SOFTWARE\Classes` ระดับเครื่องและ UsrClass.dat ของผู้ใช้: ค่าเริ่มต้นของ `<subject>\shellex\ContextMenuHandlers\<handler>` และการลงทะเบียนไลบรารี COM ที่สอดคล้องกัน

**แนวทางการสืบสวน:** ส่วนขยายเมนูคลิกขวาจากเครื่องมือบีบอัดไฟล์ ผลิตภัณฑ์ความปลอดภัย และไคลเอนต์คลาวด์พบได้ทั่วไป ให้ตรวจสอบ handler จริงและปฏิสัมพันธ์ที่เป็นทริกเกอร์ GUID ที่ระบุไม่ได้ยังคงเป็นเบาะแสที่มีประโยชน์ ไม่ใช่หลักฐานว่า DLL ไม่มีอยู่

### การ Hijack การเชื่อมโยงไฟล์ {#file_association_hijack}

**Check ID:** `file_association_hijack` · **ATT&CK ตามแคตตาล็อก:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**สิ่งที่ Juntei ตรวจสอบ:** ตรวจสอบการลงทะเบียน `shell\open\command` ของนามสกุลโดยตรงและของ ProgID ที่ระบุได้ สำหรับ `.txt`, `.pdf`, `.doc`, `.docx`, `.html`, `.htm`, `.js`, `.vbs`, `.hta`, `.exe`, `.bat`, `.cmd` และ `.ps1`

**หลักฐานที่ตรวจสอบ:**

- ที่เก็บคลาส `HKLM\SOFTWARE\Classes` ระดับเครื่องและ UsrClass.dat ของผู้ใช้: ค่าเริ่มต้นของ `<extension>` ที่เลือก และค่าเริ่มต้นของ `<extension-or-ProgID>\shell\open\command`

**แนวทางการสืบสวน:** เปรียบเทียบกับแอปพลิเคชันที่คาดว่าจะมีและตรวจสอบอาร์กิวเมนต์ของคำสั่ง นี่คือชุดนามสกุลที่กำหนดไว้ ไม่ใช่ทุกชนิดไฟล์หรือการระบุ UserChoice ที่มีผลจริงแบบเต็ม คำสั่งที่บันทึกไว้ไม่ได้พิสูจน์ว่ามีการเปิดเอกสาร

### การ Hijack Protocol Handler {#protocol_handler_hijack}

**Check ID:** `protocol_handler_hijack` · **ATT&CK ตามแคตตาล็อก:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่งเปิด URL สำหรับ http, https, mailto, ms-msdt และ ms-officecmd รวมถึงการลงทะเบียน `URL Protocol` ที่พบและการแทนที่ของผู้ใช้ที่เกี่ยวข้อง

**หลักฐานที่ตรวจสอบ:**

- ที่เก็บคลาส `HKLM\SOFTWARE\Classes` ระดับเครื่องและ UsrClass.dat ของผู้ใช้: เครื่องหมาย `URL Protocol` และค่าเริ่มต้นของ `<scheme>\shell\open\command`

**แนวทางการสืบสวน:** เบราว์เซอร์และโปรแกรมติดตั้งแอปพลิเคชันลงทะเบียน scheme อย่างถูกต้องตามปกติ ให้ติดตามคำสั่งและอาร์กิวเมนต์ ประเมินวัตถุประสงค์ที่คาดไว้ของ scheme และเชื่อมโยงกับหลักฐานการเปิด URL/โพรเซส

### ตัวจัดการส่วนขยาย COM ของ Recycle Bin {#recycle_bin_com_extension}

**Check ID:** `recycle_bin_com_extension` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการแทนที่คำสั่ง open/empty/explore ของ CLSID ของ Recycle Bin ที่ระบุไว้ และการลงทะเบียนตัวจัดการเมนูบริบท/drag-drop ระดับลูกโดยตรง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\open\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\empty\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shell\explore\command` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\ContextMenuHandlers\<child>` | `(Default)` |
| `HKLM\SOFTWARE\Classes\CLSID\{645FF040-5081-101B-9F08-00AA002F954E}\shellex\DragDropHandlers\<child>` | `(Default)` |

**แนวทางการสืบสวน:** เปรียบเทียบกับส่วนขยาย shell ที่ได้รับอนุมัติ ค่าที่รายงานบางค่าเป็นตัวระบุ COM แทนที่จะเป็นไลบรารีที่ระบุได้โดยตรง ให้สืบสวนคลาสที่อ้างอิงและการกระทำของผู้ใช้ก่อนยืนยันว่ามีการเรียกใช้โค้ด

### การ Hijack Search Protocol Handler {#search_protocol_handler}

**Check ID:** `search_protocol_handler` · **ATT&CK ตามแคตตาล็อก:** [T1546.001](https://attack.mitre.org/techniques/T1546/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน handler เริ่มต้นของ `search-ms\shell\open\command` ระดับเครื่อง/ผู้ใช้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Classes\search-ms\shell\open\command` | `(Default)` |
| `HKU\<user>\Software\Classes\search-ms\shell\open\command` | `(Default)` |

**แนวทางการสืบสวน:** การเปลี่ยน handler สามารถเปลี่ยนสิ่งที่เกิดขึ้นเมื่อเปิด URL แบบ search-ms ได้ แต่การมีอยู่ของมันไม่ใช่หลักฐานการส่ง exploit ให้เปรียบเทียบกับค่าเริ่มต้นของ Windows และสืบสวนกิจกรรม URL/โพรเซสที่จะใช้ handler นี้

### SharedTaskScheduler {#shared_task_scheduler}

**Check ID:** `shared_task_scheduler` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่า `SharedTaskScheduler` ของ Explorer ที่ไม่ว่าง ในฐานะคอมโพเนนต์ shell ที่ลงทะเบียนไว้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\SharedTaskScheduler` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ค่าอาจอ้างอิงคลาส COM แทนที่จะเป็นพาธไฟล์ปฏิบัติการ ให้ติดตามการลงทะเบียน server ของค่าเหล่านั้น และตรวจสอบพฤติกรรมของระบบปฏิบัติการ ที่มาของส่วนขยาย และหลักฐานการโหลด shell ที่เกี่ยวข้อง

### ShellExecuteHooks {#shell_execute_hooks}

**Check ID:** `shell_execute_hooks` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่า `ShellExecuteHooks` ของ Explorer ที่ไม่ว่าง ซึ่งครอบคลุมการลงทะเบียน hook การเรียกใช้ผ่าน shell รุ่นเก่า

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\CurrentVersion\Explorer\ShellExecuteHooks` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ระบุ CLSID ที่อ้างอิง และตรวจสอบว่า Windows build และนโยบายอนุญาตให้ใช้ hook หรือไม่ การลงทะเบียนรุ่นเก่าอาจไม่ได้ถูกใช้งาน ค่านี้เองไม่ใช่การเรียกใช้ผ่าน shell ที่สังเกตพบ

### การคงอยู่ผ่าน COM ด้วย Library และโฟลเดอร์ CLSID ใน Start Menu {#start_menu_com}

**Check ID:** `start_menu_com` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาไฟล์ XML ของ library `.library-ms` ที่ URL `simpleLocation` อ้างอิง
COM CLSID และไดเรกทอรีที่ตั้งชื่อเป็น `<name>.{CLSID}` ภายใต้
`Users\<user>\AppData\Roaming\Microsoft\Windows\Start Menu` ปฏิสัมพันธ์ของ Explorer กับ
ออบเจกต์เหล่านี้สามารถเปิดใช้งานโค้ด COM ได้ การตรวจสอบนี้รวมไดเรกทอรีว่างที่เก็บรวบรวมมาด้วย และไม่ติดตาม
symbolic link ไฟล์ library ทั่วไปที่อยู่นอก Start Menu และ GUID ในคอมเมนต์ XML จะไม่ถูก
รายงานโดยการตรวจสอบนี้

**หลักฐานที่ตรวจสอบ:**

- ไฟล์ `.library-ms` ใน Start Menu รายผู้ใช้ และรายการไดเรกทอรีที่ตั้งชื่อเป็น CLSID
- การลงทะเบียนคลาสที่เก็บรวบรวมมาของผู้ใช้ที่ตรงกัน และ SOFTWARE ระดับเครื่อง แยกตามสถาปัตยกรรม
- การกำหนดค่า CodeBase/AppID/บริการแบบ managed ผ่านตัวระบุ (resolver) เดียวกันกับงานตามกำหนดเวลา

**แนวทางการสืบสวน:** `Path` และ `SourceFile` ระบุอาร์ติแฟกต์ที่เป็นทริกเกอร์การเปิดใช้งาน `Value`
มีคำสั่ง server ที่เป็นผู้ต้องสงสัย หรือ CLSID หากระบุไม่ได้ `Description`
เชื่อมโยงทริกเกอร์กับ CLSID และแหล่งรีจิสทรีที่เก็บรวบรวมมา `ResolvedPath`/`Sha256` อธิบายเฉพาะ
payload ที่ระบุได้และเก็บรวบรวมมาแล้วเท่านั้น ความเชื่อมโยงกับ AppID ที่มีเพียงชื่อไฟล์ปฏิบัติการจะคงสถานะระบุไม่ได้
ทริกเกอร์อาจเป็นสิ่งที่ถูกต้อง การมีอยู่ไม่ได้พิสูจน์ว่ามีการลงชื่อเข้าใช้ การเปิดใช้งาน การเรียกใช้งาน หรือการโจมตี
ให้ตรวจสอบ URL ของ library การลงทะเบียนคลาส payload บริบทของผู้ใช้ และ log การเรียกใช้งานที่เป็นอิสระ

**ข้อควรระวังในการเก็บรวบรวม:** การเก็บรวบรวมแบบเฉพาะไฟล์อาจละเว้นไดเรกทอรีว่าง การตรวจสอบนี้ตรวจพาธ Start Menu
มาตรฐานของผู้ใช้ Start Menu ที่ถูกเปลี่ยนเส้นทางและตำแหน่ง Explorer namespace อื่น ๆ ไม่ได้ครอบคลุม
โดยการตรวจสอบนี้ ไฮฟ์ที่ขาดหายหรือการลงทะเบียนที่ระบุไม่ได้จะทำให้ coverage เป็นบางส่วน แต่ยังคงเก็บ
ผู้ต้องสงสัยไว้ สถาปัตยกรรมของงาน การยกระดับสิทธิ์ และนโยบายสามารถเปลี่ยนพฤติกรรมการค้นหา COM ที่มีผลจริงได้

**รายละเอียดเพิ่มเติม:** [ข้อเสนอต้นฉบับและเครดิตของนักวิจัย](https://github.com/last-byte/PersistenceSniper/pull/32),
[มุมมองแบบรวมของ COM รายผู้ใช้/ระดับเครื่อง](https://learn.microsoft.com/en-us/windows/win32/sysinfo/merged-view-of-hkey-classes-root),
[MITRE COM hijacking](https://attack.mitre.org/techniques/T1546/015/)

### การ Hijack COM ผ่าน TypeLib {#typelib_hijack}

**Check ID:** `typelib_hijack` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการลงทะเบียน TypeLib GUID/version/locale win32/win64 รายผู้ใช้ใน UsrClass.dat โดยเก็บพาธและค่า script moniker ไว้ ดัชนีทรัพยากรที่เป็นตัวเลขท้ายสุดจะถูกตัดออกเฉพาะสำหรับการระบุไฟล์เป้าหมายเท่านั้น

**หลักฐานที่ตรวจสอบ:**

- ค่าเริ่มต้นของ `TypeLib\<GUID>\<version>\<locale>\win32` และ `win64` ใน UsrClass.dat ที่เก็บรวบรวมมาแต่ละไฟล์

**แนวทางการสืบสวน:** type library อาจชี้ไปยัง DLL ที่ถูกต้อง ไฟล์อื่น หรือ script moniker การตรวจสอบนี้รวบรวมรายการการลงทะเบียนเหล่านี้ ให้ประเมินแอปพลิเคชันที่ได้รับผลกระทบและยืนยันการโหลด type library แทนที่จะสันนิษฐานว่าทุกพาธที่ไม่ใช่ค่าเริ่มต้นเป็นการโจมตี

## แอปพลิเคชันและ add-in ของ Office { #office-applications-and-add-ins }

Office รองรับ add-in เทมเพลต และส่วนขยายเฉพาะแอปพลิเคชันที่ถูกต้องตามปกติ การลงทะเบียนที่น่าสงสัยอาจขึ้นอยู่กับเวอร์ชันของ Office แอปพลิเคชัน การตั้งค่าการโหลด หรือปฏิสัมพันธ์ของผู้ใช้ การกำหนดค่าเพียงอย่างเดียวไม่ได้ยืนยันว่ามาโครถูกเรียกใช้งาน

### การลงทะเบียน Add-in ของ Office {#office_addins}

**Check ID:** `office_addins` · **ATT&CK ตามแคตตาล็อก:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจง add-in ของ Word, Excel, PowerPoint, Outlook และ Access ใน root ของ Office แบบไม่ระบุเวอร์ชันและเวอร์ชัน 14.0/15.0/16.0 รวมถึงที่เก็บ Click-to-Run ระดับเครื่องและ root ของผู้ใช้ที่มีการใช้งาน โดยเก็บค่า Manifest/FileName/Path ที่ระบุไว้อย่างชัดเจน หรือระบุรายการที่อิง COM ที่เข้าเกณฑ์ พร้อมบริบทของ LoadBehavior

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` และตำแหน่ง Click-to-Run ระดับเครื่องที่เทียบเท่ากันที่มีการใช้งาน
- `HKU\<user>\Software\Microsoft\Office\[<version>\]<application>\Addins\<add-in>` ใน NTUSER.DAT รวมถึงคลาส COM ระดับเครื่อง/ผู้ใช้ที่ตรงกันจาก SOFTWARE, NTUSER.DAT และ UsrClass.dat

**แนวทางการสืบสวน:** add-in เป็นซอฟต์แวร์ทางธุรกิจที่พบได้ทั่วไป ให้ตรวจทาน LoadBehavior แอปพลิเคชัน/เวอร์ชัน manifest, payload จริง และผู้จำหน่ายที่คาดว่าจะเป็น การตรวจสอบนี้ไม่ได้รับประกันว่าทุกรูปแบบ add-in ที่ติดตั้งหรือมุมมองการลงทะเบียนได้ถูกเก็บรวบรวมหรือโหลดแล้ว

### การ Hijack Add-in AI ของ Office {#office_ai_hijack}

**Check ID:** `office_ai_hijack` · **ATT&CK ตามแคตตาล็อก:** [T1137.006](https://attack.mitre.org/techniques/T1137/006/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าในตำแหน่งการลงทะเบียน Common AI ของ Office 16.0 Click-to-Run

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Office\ClickToRun\REGISTRY\MACHINE\Software\Microsoft\Office\16.0\Common\AI`: ค่าทั้งหมด

**แนวทางการสืบสวน:** เปรียบเทียบค่าและพาธที่ผิดปกติกับ Office build ที่ติดตั้ง การมีอยู่ของค่าไม่ได้แสดงว่าฟีเจอร์ใดของ Office ใช้ค่านั้น หรือพิสูจน์ว่าค่านั้นเลือกโค้ดที่เรียกใช้งานได้

### การแทนที่ DLL ของ Office {#office_dll_override}

**Check ID:** `office_dll_override` · **ATT&CK ตามแคตตาล็อก:** [T1137](https://attack.mitre.org/techniques/T1137/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `WwlibtDll` และ `PPCoreTDLL` ภายใต้ root ของ Word/PowerPoint สำหรับการตั้งค่าแบบไม่ระบุเวอร์ชันและเวอร์ชัน 14.0/15.0/16.0 รวมถึง root ของ Click-to-Run และของผู้ใช้ที่มีการใช้งาน

**หลักฐานที่ตรวจสอบ:**

- การตั้งค่า `Microsoft\Office\[<version>\]Word` และ `PowerPoint` ระดับเครื่อง/ผู้ใช้ รวมถึงโครงสร้าง Office ของ Click-to-Run ระดับเครื่องที่มีการใช้งาน: `WwlibtDll` และ `PPCoreTDLL`

**แนวทางการสืบสวน:** พิจารณาว่าแอปพลิเคชัน/build ใดรู้จักการแทนที่นี้ ตรวจสอบไลบรารีและที่มา และเชื่อมโยงกับการเปิด Office หรือการโหลด image การแทนที่ที่กำหนดค่าไว้เพียงอย่างเดียวไม่ใช่หลักฐานการเรียกใช้งาน

### เทมเพลตเริ่มต้นของ Office {#office_templates}

**Check ID:** `office_templates` · **ATT&CK ตามแคตตาล็อก:** [T1137.001](https://attack.mitre.org/techniques/T1137/001/)

**สิ่งที่ Juntei ตรวจสอบ:** รวบรวมรายการไฟล์ Normal.dotm/NormalEmail.dotm ที่เก็บรวบรวมมา และไฟล์ที่อยู่โดยตรงในโฟลเดอร์ Word Startup และ Excel XLSTART ที่รู้จัก รวมถึงโฟลเดอร์ startup ของการติดตั้ง Office

**หลักฐานที่ตรวจสอบ:**

- `Users\<user>\AppData\Roaming\Microsoft\Templates\Normal.dotm` และ `NormalEmail.dotm`
- ไฟล์ที่อยู่โดยตรงใน `Microsoft\Word\STARTUP` และ `Microsoft\Excel\XLSTART` ของผู้ใช้ภายใต้ Roaming AppData
- ไฟล์ที่อยู่โดยตรงในโฟลเดอร์ Office `<version>\STARTUP` / `XLSTART` และ `root\<version>\STARTUP` / `XLSTART` ที่รู้จัก ภายใต้ Program Files และ Program Files (x86)

**แนวทางการสืบสวน:** เทมเพลตอาจถูกต้องตามปกติหรือไม่มีเนื้อหาที่เรียกใช้งานได้เลย Juntei ไม่ได้แยกหรือเรียกใช้มาโครในการตรวจสอบนี้ ให้ตรวจสอบโครงสร้างภายในของเอกสาร นโยบายมาโคร และกิจกรรมของแอปพลิเคชัน พาธการติดตั้งระดับเครื่องไม่ได้บ่งบอกถึงโทเค็นของโพรเซส SYSTEM

### Office Test DLL {#office_test_dll}

**Check ID:** `office_test_dll` · **ATT&CK ตามแคตตาล็อก:** [T1137.002](https://attack.mitre.org/techniques/T1137/002/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าในคีย์ Office Test Special Perf จากมุมมองระดับเครื่อง/ผู้ใช้และมุมมองระดับเครื่อง 32 บิตที่ระบุไว้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Office Test\Special\Perf` | ค่าที่ไม่ว่างทั้งหมด |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Office Test\Special\Perf` | ค่าที่ไม่ว่างทั้งหมด |
| `HKU\<user>\SOFTWARE\Microsoft\Office Test\Special\Perf` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ให้ถือว่าการแทนที่ที่ไม่คาดคิดเป็นเบาะแสที่ต้องตรวจสอบเวอร์ชัน Office และ payload การตรวจสอบนี้รายงานการกำหนดค่า แทนที่จะทดสอบว่าการติดตั้ง Office ใดโหลดค่านั้นหรือไม่

### การโจมตีผ่านหน้าแรกของ Outlook {#outlook_home_page}

**Check ID:** `outlook_home_page` · **ATT&CK ตามแคตตาล็อก:** [T1137.004](https://attack.mitre.org/techniques/T1137/004/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `URL` ภายใต้ลูกของ WebView ใน Outlook ของผู้ใช้สำหรับ Office เวอร์ชัน 14.0, 15.0 และ 16.0

**หลักฐานที่ตรวจสอบ:**

- `HKU\<user>\Software\Microsoft\Office\<14.0|15.0|16.0>\Outlook\WebView\<folder>\URL` ใน NTUSER.DAT

**แนวทางการสืบสวน:** หน้าแรกของโฟลเดอร์อาจถูกต้องตามปกติ และพฤติกรรมแตกต่างกันตามเวอร์ชันของ Outlook และอัปเดตความปลอดภัย ให้ตรวจสอบ URL/เนื้อหาและนโยบายที่มีผล จากนั้นเชื่อมโยงกับหลักฐานของ Outlook/เว็บ/โพรเซส

### การ Hijack VBA Monitor DLL {#vba_monitors}

**Check ID:** `vba_monitors` · **ATT&CK ตามแคตตาล็อก:** [T1137](https://attack.mitre.org/techniques/T1137/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า InprocServer32 ของ CLSID ที่เกี่ยวข้องกับ VBA monitor สองรายการที่มีการใช้งาน ในที่เก็บคลาสระดับเครื่องและรายผู้ใช้

**หลักฐานที่ตรวจสอบ:**

- `Classes\CLSID\{13B4E945-2B11-4B60-94A9-B6CDE52F6F93}\InprocServer32` และ `{0002E157-0000-0000-C000-000000000046}` ใน SOFTWARE ระดับเครื่อง รวมถึงการลงทะเบียนที่เทียบเท่ากันใน UsrClass.dat ของผู้ใช้

**แนวทางการสืบสวน:** เปรียบเทียบไลบรารีที่ถูกเลือกกับสภาพแวดล้อม Office/VBA ที่ติดตั้ง นี่คือการตรวจสอบ CLSID แบบเจาะจง ไม่ใช่รายการส่วนขยาย VBA ทั้งหมดหรือหลักฐานว่ามาโครได้ทำงาน

## ดีบักเกอร์ ตัวจัดการการแครช และ compatibility shim { #debuggers-crash-handlers-and-compatibility-shims }

ดีบักเกอร์หรือตัวจัดการข้อผิดพลาดสามารถทำงานเมื่อแอปพลิเคชันเริ่มทำงาน แครช ค้าง หรือปิดตัวลง ฐานข้อมูลความเข้ากันได้ของแอปพลิเคชันก็อาจส่งผลต่อการเริ่มทำงานได้เช่นกัน ให้สืบสวนการตั้งค่าที่เป็นเงื่อนไขเบื้องต้นและโปรแกรมที่เป็นทริกเกอร์ควบคู่ไปกับคำสั่งที่รายงาน

### การ Hijack ดีบักเกอร์ AeDebug {#ae_debug}

**Check ID:** `ae_debug` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง `Debugger` ของ AeDebug แบบดั้งเดิมและ 32 บิต ซึ่งใช้โดยการกำหนดค่าการดีบักการแครชแบบ just-in-time

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebug` | `Debugger` |

**แนวทางการสืบสวน:** ซอฟต์แวร์พัฒนา/ดีบักอาจตั้งค่านี้อย่างถูกต้องตามปกติ ให้ตรวจสอบคำสั่งดีบักเกอร์ฉบับเต็มและเงื่อนไขเบื้องต้นของทริกเกอร์การแครช จากนั้นเชื่อมโยงกับเหตุการณ์ข้อผิดพลาดของแอปพลิเคชันและโพรเซส

### ดีบักเกอร์ Protected Process ของ AeDebug {#ae_debug_protected}

**Check ID:** `ae_debug_protected` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านคำสั่ง `Debugger` ของ AeDebugProtected แบบดั้งเดิมและ 32 บิต สำหรับพื้นผิวการดีบัก protected process

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\AeDebugProtected` | `Debugger` |

**แนวทางการสืบสวน:** พิจารณาว่า build ของระบบปฏิบัติการและนโยบายการป้องกันสามารถใช้คำสั่งที่กำหนดค่าไว้ได้หรือไม่ ดีบักเกอร์ที่บันทึกไว้ไม่ใช่หลักฐานว่า protected process แครชหรือคำสั่งได้ทำงาน

### ดีบักเกอร์ใน Image File Execution Options {#ifeo_debugger}

**Check ID:** `ifeo_debugger` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาค่า `Debugger` ในการลงทะเบียน Image File Execution Options แบบดั้งเดิม/32 บิต รวมถึงคีย์ตัวกรองที่ซ้อนกัน

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\Debugger` รวมถึงคีย์ตัวกรองที่ซ้อนกันและมุมมอง `Wow6432Node` ระดับเครื่อง

**แนวทางการสืบสวน:** ดีบักเกอร์อาจถูกเลือกเมื่อมีการเปิด image ที่ระบุชื่อ ให้ตรวจสอบขอบเขตของ image/ตัวกรองและอาร์กิวเมนต์ของดีบักเกอร์ เครื่องมือของนักพัฒนาก็ใช้ IFEO ได้เช่นกัน ให้ติดตามการเปิดเป้าหมายเพื่อยืนยันการเรียกใช้งาน

### IFEO Delegated NTDLL {#ifeo_delegated_ntdll}

**Check ID:** `ifeo_delegated_ntdll` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหาการแทนที่ `VerifierDlls` ในการลงทะเบียน Image File Execution Options แบบดั้งเดิม/32 บิต รวมถึงคีย์ที่ซ้อนกัน

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Image File Execution Options\<image>\...\VerifierDlls` รวมถึงคีย์ที่ซ้อนกันและมุมมอง `Wow6432Node` ระดับเครื่อง

**แนวทางการสืบสวน:** เงื่อนไขของการตรวจสอบนี้คือค่า VerifierDlls ที่กำหนดค่าไว้ ไม่ได้พิสูจน์ว่าเงื่อนไขเบื้องต้นด้าน verifier/delegation ทั้งหมดถูกเปิดใช้งาน ให้ตรวจทานแฟล็กที่เกี่ยวข้อง เวอร์ชันของ Windows และ payload ก่อนอนุมานว่ามีเส้นทางการ inject ที่ใช้งานได้

### Silent Process Exit Monitor {#ifeo_silent_process_exit}

**Check ID:** `ifeo_silent_process_exit` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** ค้นหา `MonitorProcess` ในการลงทะเบียน SilentProcessExit แบบดั้งเดิม/32 บิต รวมถึงคีย์ที่ซ้อนกัน

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\<image>\...\MonitorProcess` รวมถึงคีย์ที่ซ้อนกันและมุมมอง `Wow6432Node` ระดับเครื่อง

**แนวทางการสืบสวน:** การตรวจติดตาม silent exit ยังต้องใช้การกำหนดค่า reporting/IFEO ที่เกี่ยวข้อง ให้ตรวจสอบการตั้งค่าเหล่านั้นและเหตุการณ์การปิดตัวของ image ที่ถูกตรวจติดตาม เครื่องมือวินิจฉัยที่ถูกต้องอาจอธิบาย monitor นี้ได้

### Application Shimming {#installed_sdb}

**Check ID:** `installed_sdb` · **ATT&CK ตามแคตตาล็อก:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DatabasePath` จากการลงทะเบียน InstalledSDB ระดับลูกโดยตรง สำหรับฐานข้อมูลความเข้ากันได้ของแอปพลิเคชันที่ติดตั้ง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\<child>` | `DatabasePath` |

**แนวทางการสืบสวน:** การแก้ไขด้านความเข้ากันได้อาจถูกต้องตามปกติ ให้ตรวจสอบเนื้อหาของ SDB ไฟล์ปฏิบัติการที่เป็นเป้าหมาย และประวัติการติดตั้ง การลงทะเบียนฐานข้อมูลไม่ได้ระบุ shim ภายใน หรือพิสูจน์ว่าแอปพลิเคชันที่ตรงกันได้ทำงาน

### การ Hijack ดีบักเกอร์ LSM {#lsm_debugger}

**Check ID:** `lsm_debugger` · **ATT&CK ตามแคตตาล็อก:** [T1546.012](https://attack.mitre.org/techniques/T1546/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `MonitorProcess` เฉพาะภายใต้ SilentProcessExit สำหรับ `lsm.exe` ในมุมมองระดับเครื่องแบบดั้งเดิมและ 32 บิต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows NT\CurrentVersion\SilentProcessExit\lsm.exe` | `MonitorProcess` |

**แนวทางการสืบสวน:** การตรวจสอบนี้ซ้อนทับกับรายการ SilentProcessExit ที่กว้างกว่า ให้ยืนยันแฟล็กที่เป็นเงื่อนไขเบื้องต้นและการปิดตัวของ LSM ที่เกี่ยวข้องก่อนถือว่าการกำหนดค่าเป็นการคงอยู่ที่ถูกเรียกใช้แล้ว และนับรายการเดียวกันเพียงครั้งเดียวเมื่อประเมินผลการตรวจพบที่แตกต่างกัน

### MiniDump Auxiliary DLLs {#minidump_auxiliary_dlls}

**Check ID:** `minidump_auxiliary_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าที่มีชื่อและไม่ว่างใน MiniDumpAuxiliaryDlls โดยเก็บชื่อค่าไว้เป็นบริบทของโมดูลทริกเกอร์ และเก็บข้อมูลของค่าไว้เป็นเป้าหมายที่ถูกเลือก

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\MiniDumpAuxiliaryDlls`: ชื่อค่าที่เป็นโมดูลทริกเกอร์และข้อมูลเป้าหมาย

**แนวทางการสืบสวน:** ตรวจสอบทั้งโมดูลทริกเกอร์และ auxiliary DLL เครื่องมือแครช/minidump อาจอธิบายรายการได้ และการลงทะเบียนไม่ได้แสดงว่าการทำ dump ได้โหลดไลบรารีนั้น

### Application Shimming {#shim_custom}

**Check ID:** `shim_custom` · **ATT&CK ตามแคตตาล็อก:** [T1546.011](https://attack.mitre.org/techniques/T1546/011/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่าที่มีชื่อเป็น `.sdb` ภายใต้คีย์แอปพลิเคชัน Custom ของ AppCompatFlags ระดับลูกโดยตรง โดยเก็บความเชื่อมโยงระหว่างแอปพลิเคชันกับฐานข้อมูลไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<application>`: ชื่อค่าที่ลงท้ายด้วย `.sdb`

**แนวทางการสืบสวน:** จับคู่ความเชื่อมโยงเหล่านี้กับ InstalledSDB และฐานข้อมูลความเข้ากันได้จริง ความเชื่อมโยงของชื่อไฟล์เพียงอย่างเดียวไม่ได้เปิดเผยพฤติกรรมของฐานข้อมูล หรือแสดงว่ามีการเปิดแอปพลิเคชันที่ตรงกัน

### การ Hijack ดีบักเกอร์ WER {#wer_debugger}

**Check ID:** `wer_debugger` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Debugger` ของ Windows Error Reporting แบบดั้งเดิม/32 บิต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `Debugger` |

**แนวทางการสืบสวน:** ซอฟต์แวร์วินิจฉัยที่ถูกต้องอาจกำหนดค่าดีบักเกอร์ ให้สืบสวนคำสั่งฉบับเต็มและเหตุการณ์การแครชที่เกี่ยวข้อง การลงทะเบียนไม่ใช่หลักฐานว่า Windows Error Reporting ได้เริ่มคำสั่งนั้น

### การ Hijack ดีบักเกอร์ Hangs ของ WER {#wer_hangs}

**Check ID:** `wer_hangs` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `Debugger` ของ Hangs ใน Windows Error Reporting แบบดั้งเดิม/32 บิต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting\Hangs` | `Debugger` |

**แนวทางการสืบสวน:** ตรวจสอบว่าขั้นตอนการจัดการอาการค้างจะเรียกใช้การตั้งค่านี้หรือไม่ และตรวจสอบโปรแกรมเป้าหมาย ให้เชื่อมโยงกับเรคคอร์ดอาการค้างของแอปพลิเคชันและโพรเซส แทนที่จะถือว่าการเขียนรีจิสทรีเป็นการเรียกใช้ดีบักเกอร์

### การ Hijack ReflectDebugger ของ WER {#wer_reflect_debugger}

**Check ID:** `wer_reflect_debugger` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `ReflectDebugger` ของ Windows Error Reporting แบบดั้งเดิม/32 บิต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\Windows\Windows Error Reporting` | `ReflectDebugger` |

**แนวทางการสืบสวน:** เปรียบเทียบกับการกำหนดค่าการวินิจฉัยที่ได้รับอนุมัติ และยืนยันความเกี่ยวข้องกับเวอร์ชัน Windows/ทริกเกอร์ ตรวจสอบอาร์กิวเมนต์ของคำสั่งและหาหลักฐานยืนยันจากการแครช/โพรเซสก่อนระบุว่ามีการเรียกใช้งาน

### WER Runtime Exception Modules {#wer_runtime_exception}

**Check ID:** `wer_runtime_exception` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงชื่อค่าภายใต้ RuntimeExceptionHelperModules แบบดั้งเดิม/32 บิต ในที่นี้พาธของไลบรารีอยู่ในชื่อค่า ไม่จำเป็นต้องอยู่ในข้อมูลของค่า

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows\Windows Error Reporting\RuntimeExceptionHelperModules` และมุมมอง `Wow6432Node` ระดับเครื่อง: ชื่อค่า

**แนวทางการสืบสวน:** ตรวจสอบโมดูลที่ระบุชื่อและบริบทของแอปพลิเคชัน/runtime ค่าตัวเลขเล็ก ๆ ในข้อมูลไม่ใช่พาธของ payload และการลงทะเบียนเพียงอย่างเดียวไม่ได้พิสูจน์ว่า helper ถูกโหลด

## การตั้งค่า .NET runtime และโปรไฟเลอร์ { #net-runtime-and-profiler-settings }

runtime แบบ managed สามารถโหลดโปรไฟเลอร์ startup assembly หรือไลบรารีดีบักก่อนโค้ดของแอปพลิเคชันได้ การตั้งค่าเหล่านี้ยังถูกใช้โดยนักพัฒนาและผลิตภัณฑ์การตรวจติดตามด้วย ให้ตรวจสอบ runtime สถาปัตยกรรมของโพรเซส แฟล็กการเปิดใช้งาน และโพรเซสที่เพิ่งเริ่มทำงานสามารถสืบทอดการตั้งค่านั้นได้หรือไม่

### AppDomainManager Injection {#appdomain_manager}

**Check ID:** `appdomain_manager` · **ATT&CK ตามแคตตาล็อก:** [T1574.014](https://attack.mitre.org/techniques/T1574/014/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านตัวแปรสภาพแวดล้อม `APPDOMAIN_MANAGER_ASM` และ `APPDOMAIN_MANAGER_TYPE` ระดับเครื่อง/ผู้ใช้ ซึ่งครอบคลุมการเลือก assembly และ type สำหรับ .NET AppDomain manager แบบกำหนดเอง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` | `APPDOMAIN_MANAGER_TYPE` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_ASM` |
| `HKU\<user>\Environment` | `APPDOMAIN_MANAGER_TYPE` |

**แนวทางการสืบสวน:** แอปพลิเคชันอาจตั้งใจใช้ manager แบบกำหนดเอง ให้ระบุ assembly และ type ร่วมกัน และตรวจสอบความเข้ากันได้ของ runtime และการสืบทอดของโพรเซส ทั้งตัวแปรเพียงตัวเดียวหรือ timestamp ของคีย์ไม่ได้พิสูจน์ว่าแอปพลิเคชัน managed ได้โหลดมัน

### การ Hijack โปรไฟเลอร์ .NET CLR {#cor_profiler}

**Check ID:** `cor_profiler` · **ATT&CK ตามแคตตาล็อก:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าสภาพแวดล้อมการทำโปรไฟล์ COR ระดับเครื่อง/ผู้ใช้ และรายการที่มีชื่อตรงกันใน multi-string Environment ของแต่ละบริการ โดยเก็บแฟล็กการเปิดใช้งาน ตัวระบุโปรไฟเลอร์ และพาธเฉพาะสถาปัตยกรรมไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` และ `Environment` ใน NTUSER.DAT ของผู้ใช้แต่ละคนที่เก็บรวบรวมมา
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: การตั้งค่า COR ที่มีชื่อใน multi-string

**แนวทางการสืบสวน:** โปรไฟเลอร์รองรับการตรวจติดตามที่ถูกต้องตามปกติ แฟล็กการเปิดใช้งาน CLSID และพาธ DLL เป็นการกำหนดค่าคนละส่วนกัน ให้ยืนยันการเปิดใช้งานที่มีผลจริงและโพรเซส .NET ที่เกี่ยวข้อง แทนที่จะถือว่าแต่ละแถวเป็นโปรไฟเลอร์ที่ถูกเรียกใช้แล้ว

### การ Hijack โปรไฟเลอร์ .NET Core CLR {#coreclr_profiler}

**Check ID:** `coreclr_profiler` · **ATT&CK ตามแคตตาล็อก:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าสภาพแวดล้อมการทำโปรไฟล์ CORECLR ระดับเครื่อง/ผู้ใช้ที่สอดคล้องกัน และรายการ Environment ของบริการที่ตรงกัน รวมถึงพาธเฉพาะสถาปัตยกรรม

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment` และ `Environment` ใน NTUSER.DAT ของผู้ใช้แต่ละคนที่เก็บรวบรวมมา
- `HKLM\SYSTEM\<ControlSet>\Services\<service>\Environment`: การตั้งค่า CORECLR ที่มีชื่อใน multi-string

**แนวทางการสืบสวน:** ตรวจสอบ .NET runtime สถาปัตยกรรม แฟล็กการเปิดใช้งาน และไลบรารี ผลิตภัณฑ์ด้าน observability ที่ได้รับอนุมัติอาจกำหนดค่าเหล่านี้ และโพรเซสที่มีอยู่แล้วอาจไม่สืบทอดสภาพแวดล้อมที่เพิ่งเปลี่ยน

### การ Hijack .NET DbgManagedDebugger {#dotnet_dbg_managed_debugger}

**Check ID:** `dotnet_dbg_managed_debugger` · **ATT&CK ตามแคตตาล็อก:** [T1546](https://attack.mitre.org/techniques/T1546/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่า `DbgManagedDebugger` ของ `.NETFramework` ระดับเครื่อง/ผู้ใช้ที่ระบุไว้ รวมถึงพาธ 32 บิตระดับเครื่องที่ระบุอย่างชัดเจน

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `DbgManagedDebugger` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `DbgManagedDebugger` |

**แนวทางการสืบสวน:** พิจารณาว่า .NET runtime/นโยบายการดีบักที่เกี่ยวข้องใช้คำสั่งนี้หรือไม่ เครื่องมือของนักพัฒนาอาจอธิบายค่านี้ได้ ให้เชื่อมโยงกับการแครชของโค้ด managed และการสร้างโพรเซสเพื่อยืนยันการเรียกใช้งาน

### คีย์รีจิสทรี COR_PROFILER ของ .NET Framework {#dotnet_framework_profiler}

**Check ID:** `dotnet_framework_profiler` · **ATT&CK ตามแคตตาล็อก:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `COR_PROFILER`, `COR_PROFILER_PATH` และ `COR_ENABLE_PROFILING` จากมุมมองรีจิสทรี `.NETFramework` ที่ระบุไว้ ซึ่งเสริมการตรวจสอบโปรไฟเลอร์ที่อิงตัวแปรสภาพแวดล้อม

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_PROFILER_PATH` |
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKLM\SOFTWARE\Wow6432Node\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |
| `HKU\<user>\SOFTWARE\Microsoft\.NETFramework` | `COR_ENABLE_PROFILING` |

**แนวทางการสืบสวน:** ประเมินค่าเหล่านี้ร่วมกันและพิจารณาว่า runtime ยอมรับตำแหน่งรีจิสทรีนั้นหรือไม่ เครื่องมือทำโปรไฟล์ที่กำหนดค่าไว้อาจถูกต้องตามปกติ อย่าตีความแฟล็กการเปิดใช้งานเพียงอย่างเดียวว่าเป็นการโหลด DLL ที่ระบุได้

### DOTNET_STARTUP_HOOKS {#dotnet_startup_hooks}

**Check ID:** `dotnet_startup_hooks` · **ATT&CK ตามแคตตาล็อก:** [T1574.012](https://attack.mitre.org/techniques/T1574/012/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่าสภาพแวดล้อม `DOTNET_STARTUP_HOOKS` ระดับเครื่อง/ผู้ใช้ ซึ่งสามารถเลือก assembly ของ startup hook สำหรับ .NET runtime ที่รองรับ

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\Environment\DOTNET_STARTUP_HOOKS` และ `Environment\DOTNET_STARTUP_HOOKS` ใน NTUSER.DAT ของผู้ใช้แต่ละคน

**แนวทางการสืบสวน:** ตรวจสอบค่าทั้งหมดเพื่อหา assembly หลายรายการ และยืนยันการรองรับของ runtime การตรวจสอบนี้ไม่ได้แจกแจงสภาพแวดล้อม startup hook เฉพาะบริการ หรือพิสูจน์ว่าโพรเซสใดได้สืบทอดและใช้ค่านั้น

### Known Managed Debugging DLLs {#known_managed_debugging_dlls}

**Check ID:** `known_managed_debugging_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการลงทะเบียน `KnownManagedDebuggingDlls` ของ `.NETFramework` ในฐานะการแทนที่ไลบรารีของดีบักเกอร์ managed ที่เป็นไปได้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\.NETFramework` | `KnownManagedDebuggingDlls` |

**แนวทางการสืบสวน:** เปรียบเทียบกับเครื่องมือพัฒนา/runtime ที่ติดตั้ง และระบุดีบักเกอร์ที่เป็นทริกเกอร์ รายการในรีจิสทรีไม่ได้พิสูจน์ว่ามีเซสชันดีบักหรือการโหลดไลบรารี

## การแทนที่การค้นหา DLL และไฟล์ปฏิบัติการ { #dll-and-executable-lookup-overrides }

การตรวจสอบเหล่านี้ตรวจตำแหน่งไลบรารีที่ระบุอย่างชัดเจน หรือการตั้งค่าที่มีอิทธิพลต่อวิธีที่คอมโพเนนต์ค้นหาโค้ด การแทนที่พาธหรือนโยบายการค้นหาที่อ่อนลงเป็นเบาะแสสำหรับการสืบสวน แต่การเรียกใช้งานยังต้องมี payload และคอมโพเนนต์ทริกเกอร์ที่เข้ากันได้

### AppCert DLLs {#appcert_dlls}

**Check ID:** `appcert_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1546.009](https://attack.mitre.org/techniques/T1546/009/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงการลงทะเบียน AppCertDlls ของ Session Manager ที่ไม่ว่าง ซึ่งเป็นพื้นผิวส่วนขยายไลบรารีที่เกี่ยวข้องกับ API การสร้างโพรเซส

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\AppCertDlls` | ค่าที่ไม่ว่างทั้งหมด |

**แนวทางการสืบสวน:** ตรวจสอบแต่ละไลบรารีและที่มาของการติดตั้ง พิจารณาความเกี่ยวข้องและการโหลดขณะทำงานแยกกัน การสร้างโพรเซสตามปกติในไทม์ไลน์ไม่ใช่หลักฐานในตัวเองว่า DLL ที่กำหนดค่าไว้ถูกโหลด

### AppInit DLLs {#appinit_dlls}

**Check ID:** `appinit_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1546.010](https://attack.mitre.org/techniques/T1546/010/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `AppInit_DLLs` แบบดั้งเดิม/32 บิต แยกรายการไลบรารี และเพิ่มบริบทของ `LoadAppInit_DLLs` และ `RequireSignedAppInit_DLLs` ที่มีให้แก่ผู้ต้องสงสัยแต่ละรายการ

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Windows`: `AppInit_DLLs`, `LoadAppInit_DLLs` และ `RequireSignedAppInit_DLLs` รวมถึงมุมมอง `Wow6432Node` ระดับเครื่อง

**แนวทางการสืบสวน:** แฟล็ก LoadAppInit_DLLs ที่เป็นศูนย์จะถูกระบุอย่างชัดเจนว่าไม่ทำงาน เวอร์ชันของ Windows, Secure Boot และข้อกำหนดการลงนามก็ส่งผลต่อการโหลดเช่นกัน ให้ตรวจทานการตั้งค่าที่มีผลจริงก่อนกล่าวอ้างว่า AppInit DLL ถูกเรียกใช้งาน

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/secure-boot-and-appinit-dlls)

### การแทนที่ AutodialDLL {#autodial_dll}

**Check ID:** `autodial_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการแทนที่ `AutodialDLL` ใน Parameters ของ WinSock2

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters` | `AutodialDLL` |

**แนวทางการสืบสวน:** ตรวจสอบฟีเจอร์เครือข่ายและเวอร์ชัน Windows ต้นทางที่ใช้การตั้งค่านี้ ตรวจทาน DLL จริงและกิจกรรมเครือข่าย/โพรเซส การแทนที่เพียงอย่างเดียวไม่ได้ยืนยันว่ามีการโหลดที่ใช้งานอยู่หรือเป็นอันตราย

### CHM Helper DLL {#chm_helper_dll}

**Check ID:** `chm_helper_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Location` ภายใต้ HtmlHelp Author ซึ่งเป็นพื้นผิวการแทนที่ไลบรารี/ตำแหน่งที่เกี่ยวข้องกับระบบช่วยเหลือ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\HtmlHelp Author` | `Location` |

**แนวทางการสืบสวน:** พิจารณาว่าคอมโพเนนต์ช่วยเหลือที่เกี่ยวข้องใช้ค่านี้บนระบบนี้หรือไม่ และตรวจสอบไฟล์ที่ถูกเลือก การแทนที่ในรีจิสทรีไม่ใช่หลักฐานว่ามีการเปิดเอกสาร CHM

### การแทนที่ DLL ของ NL6 ContentIndex {#content_index_dll}

**Check ID:** `content_index_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574](https://attack.mitre.org/techniques/T1574/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DLLOverridePath` จากการลงทะเบียน ContentIndex Language ระดับลูกโดยตรงใน control set ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\ContentIndex\Language\<language>\DLLOverridePath`

**แนวทางการสืบสวน:** คอมโพเนนต์ภาษา/การค้นหาอาจแตกต่างกันตาม Windows build ให้ตรวจสอบไลบรารีและคอมโพเนนต์ที่ติดตั้งที่ตรงกัน และติดตามโพรเซสการทำดัชนีจริงเพื่อยืนยันการโหลด

### Crypto ExpoOffload DLL {#crypto_expo_offload}

**Check ID:** `crypto_expo_offload` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `ExpoOffload` ของ Cryptography Offload ในฐานะผู้ต้องสงสัยการแทนที่ไลบรารีการเข้ารหัส

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Cryptography\Offload` | `ExpoOffload` |

**แนวทางการสืบสวน:** ยืนยันความเกี่ยวข้องกับฟีเจอร์/build และสืบสวนที่มาของไลบรารีที่ถูกเลือก อย่าถือว่าการมีอยู่ของค่าที่ไม่ค่อยพบเป็นหลักฐานการขโมยข้อมูลประจำตัวหรือโค้ดที่ถูกเรียกใช้งาน

### Direct3D Software Rasterizer DLL {#direct3d_dll}

**Check ID:** `direct3d_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการแทนที่ไลบรารี `SoftwareRasterizer` ของ Direct3D Drivers

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Direct3D\Drivers` | `SoftwareRasterizer` |

**แนวทางการสืบสวน:** ซอฟต์แวร์กราฟิกและซอฟต์แวร์ด้านความเข้ากันได้สามารถส่งผลต่อการกำหนดค่า rasterizer ได้ ให้ตรวจสอบ DLL และกิจกรรมของแอปพลิเคชัน Direct3D ที่เข้ากันได้ การบันทึกพาธไม่ได้พิสูจน์ว่า DLL ถูกโหลด

### การลดความเข้มงวดของลำดับการค้นหา DLL {#dll_search_mode}

**Check ID:** `dll_search_mode` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** รายงาน `SafeDllSearchMode` หรือ `CWDIllegalInDllSearch` เฉพาะเมื่อค่าใน Session Manager ที่ใช้งานอยู่แปลงได้เป็นศูนย์ ค่าเหล่านี้เป็นเงื่อนไขของนโยบายการค้นหา ไม่ใช่ payload ที่ถูกเลือก

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager`: `SafeDllSearchMode` และ `CWDIllegalInDllSearch`

**แนวทางการสืบสวน:** นโยบายการค้นหาที่อ่อนลงอาจทำให้การโจมตีผ่านลำดับการค้นหาที่แยกต่างหากง่ายขึ้น แต่ไม่ได้ระบุ DLL ที่ถูกวางไว้หรือพิสูจน์การเรียกใช้งาน ค่าที่ไม่เป็นศูนย์หรือไม่มีอยู่จะไม่ถูกส่งออกโดยการตรวจสอบนี้ ให้ประเมินค่าเริ่มต้นที่มีผลและนโยบายแยกต่างหาก

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-security)

### การยกเว้นจาก KnownDLLs {#exclude_from_known_dlls}

**Check ID:** `exclude_from_known_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `ExcludeFromKnownDlls` ที่ไม่ว่างจาก Session Manager ในฐานะการเปลี่ยนแปลงพฤติกรรมการค้นหา DLL แบบป้องกัน/ที่รู้จัก

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\ExcludeFromKnownDlls`

**แนวทางการสืบสวน:** การยกเว้นระบุเงื่อนไขการค้นหา ไม่ใช่ไลบรารีทดแทนเอง ให้หาแอปพลิเคชันที่ได้รับผลกระทบและตำแหน่งค้นหา DLL ที่เป็นไปได้ จากนั้นยืนยันการโหลด image ที่ไม่เป็นมาตรฐาน

**รายละเอียดเพิ่มเติม:** [เอกสารของ Microsoft](https://learn.microsoft.com/en-us/windows/win32/dlls/dynamic-link-library-search-order)

### การแทนที่ DLL ของ hhctrl.ocx {#hhctrl_ocx_dll}

**Check ID:** `hhctrl_ocx_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการแทนที่ InprocServer32 ในที่เก็บคลาสระดับเครื่องแบบดั้งเดิม/32 บิตและของผู้ใช้ที่รองรับ สำหรับ CLSID ของ HTML Help control ที่มีการใช้งาน

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKLM\SOFTWARE\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\SOFTWARE\Classes\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |
| `HKU\<user>\Software\Classes\Wow6432Node\CLSID\{adb880a6-d8ff-11cf-9377-00aa003b7a11}\InprocServer32` | `(Default)` |

**แนวทางการสืบสวน:** พิจารณาว่าแอปพลิเคชันและสถาปัตยกรรมใดเปิดใช้งาน control นี้ ตรวจสอบ DLL ที่ลงทะเบียนไว้และกิจกรรมที่เกี่ยวข้องกับระบบช่วยเหลือ การลงทะเบียน COM ไม่ได้พิสูจน์ว่า HTML Help ได้แสดงผลเนื้อหา

### Known DLLs {#known_dlls}

**Check ID:** `known_dlls` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจงค่าที่มีชื่อในคีย์ KnownDLLs ของ Session Manager ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Control\Session Manager\KnownDLLs`: ค่าที่มีชื่อ

**แนวทางการสืบสวน:** รายการจำนวนมากเป็นไลบรารีของ Windows ที่คาดว่าจะมี และบางค่าเป็นการกำหนดค่าสนับสนุนแทนที่จะเป็นพาธของ payload ให้เปรียบเทียบคีย์ทั้งหมดกับ build ที่ตรงกันและเชื่อถือได้ การตรวจสอบนี้ไม่ได้สแกนโครงสร้าง KnownDLLs เฉพาะสถาปัตยกรรมทางเลือกทุกรายการ

### LSA Extensions DLL {#lsa_extensions}

**Check ID:** `lsa_extensions` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Extensions` จากการกำหนดค่า LsaSrv ของ LsaExtensionConfig

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\LsaExtensionConfig\LsaSrv` | `Extensions` |

**แนวทางการสืบสวน:** ซอฟต์แวร์ความปลอดภัยสามารถขยายคอมโพเนนต์การยืนยันตัวตนได้ ให้ตรวจสอบไลบรารีจริงและความเข้ากันได้กับเวอร์ชัน Windows ต้นทาง จากนั้นหาหลักฐานการโหลด image ใน LSASS หรือหลักฐานการใช้งานอิสระอื่น ๆ

### การแทนที่พาธ DLL ของ MAPI32 {#mapi32_dll_path}

**Check ID:** `mapi32_dll_path` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `DLLPathEx` และ `DLLPath` ภายใต้การลงทะเบียน Mail client ระดับลูกโดยตรงในมุมมองระดับเครื่องแบบดั้งเดิมและ 32 บิต

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPathEx` |
| `HKLM\SOFTWARE\Clients\Mail\<child>` | `DLLPath` |
| `HKLM\SOFTWARE\Wow6432Node\Clients\Mail\<child>` | `DLLPath` |

**แนวทางการสืบสวน:** ไคลเอนต์อีเมลลงทะเบียน MAPI provider ที่ถูกต้องตามปกติ ให้พิจารณาว่าไคลเอนต์/provider ใดถูกเลือกและตรวจสอบไลบรารีจริง provider ที่ติดตั้งไว้ไม่ได้พิสูจน์ว่าแอปพลิเคชันใช้ MAPI ระหว่างเหตุการณ์

### MSDTC XA DLL {#msdtc_xa_dll}

**Check ID:** `msdtc_xa_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `OracleXaLib` และ `OracleOciLib` จากการกำหนดค่า MTxOCI ของ MSDTC

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleXaLib` |
| `HKLM\SOFTWARE\Microsoft\MSDTC\MTxOCI` | `OracleOciLib` |

**แนวทางการสืบสวน:** การเชื่อมต่อกับฐานข้อมูลอาจต้องใช้ไลบรารีเหล่านี้อย่างถูกต้องตามปกติ ให้ตรวจสอบการกำหนดค่า transaction coordinator และซอฟต์แวร์ฐานข้อมูลที่ติดตั้ง และเชื่อมโยงกับการใช้งานบริการ/โมดูล แทนที่จะสันนิษฐานว่าพาธที่กำหนดค่าไว้ได้ทำงาน

### การแทนที่ DLL ของ NLDP {#nldp_dll}

**Check ID:** `nldp_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่าการกำหนดค่า `DllOverridePath` ของ NlsData

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\NlsData` | `DllOverridePath` |

**แนวทางการสืบสวน:** ตรวจทานไลบรารีเป้าหมาย Windows build และคอมโพเนนต์ภาษาธรรมชาติที่อาจใช้การแทนที่นี้ ค่านี้ไม่ได้แสดงว่ามีการทำดัชนีหรือการประมวลผลภาษาเกิดขึ้น

### RDP TestDVCPlugin DLL {#rdp_test_dvc_plugin}

**Check ID:** `rdp_test_dvc_plugin` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการตั้งค่าไลบรารี `TestDVCPlugin` ของ Terminal Server Client ระดับเครื่อง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Terminal Server Client` | `TestDVCPlugin` |

**แนวทางการสืบสวน:** สืบสวน test plugin ที่ไม่คุ้นเคยและพฤติกรรมเฉพาะเวอร์ชัน การยืนยันการเรียกใช้งานต้องมีเซสชันไคลเอนต์ที่เข้ากันได้และการโหลดไลบรารี ค่ารีจิสทรีเพียงอย่างเดียวไม่ได้แสดงทั้งสองอย่าง

### การแทนที่ DLL ของ Search Indexer {#search_indexer_dll}

**Check ID:** `search_indexer_dll` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านการแทนที่ `DllPath` ของ Windows Search จาก SOFTWARE

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SOFTWARE\Microsoft\Windows Search` | `DllPath` |

**แนวทางการสืบสวน:** ระบุคอมโพเนนต์/เวอร์ชันของ Search ที่ตรงกันและตรวจสอบไฟล์เป้าหมาย บริการ Windows Search ที่ไม่ได้ทำงานหรือการตั้งค่าที่ไม่ถูกใช้ แตกต่างจากการโหลดไลบรารีที่เป็นอันตรายที่สังเกตพบ

### Winsock AutoProxy DLL {#winsock_auto_proxy}

**Check ID:** `winsock_auto_proxy` · **ATT&CK ตามแคตตาล็อก:** [T1574.001](https://attack.mitre.org/techniques/T1574/001/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `LibraryPath` ภายใต้การลงทะเบียน Catalog_Entries และ Catalog_Entries64 ของ NameSpace_Catalog5 ระดับลูกโดยตรง

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries\<child>` | `LibraryPath` |
| `HKLM\SYSTEM\<ControlSet>\Services\WinSock2\Parameters\NameSpace_Catalog5\Catalog_Entries64\<child>` | `LibraryPath` |

**แนวทางการสืบสวน:** provider ด้านเครือข่ายอาจถูกต้องตามปกติ ให้ตรวจทานทั้งสองสถาปัตยกรรม ที่มาของ provider และหลักฐานของแอปพลิเคชันเครือข่าย/การโหลด image ที่เกี่ยวข้อง การค้นหา namespace provider แบบเจาะจงนี้ไม่ได้ครอบคลุมส่วนขยาย Winsock ทุกรูปแบบที่เป็นไปได้

## บัญชีและการมองเห็นในการลงชื่อเข้าใช้ { #accounts-and-sign-in-visibility }

เรคคอร์ดบัญชีภายในเครื่องและการตั้งค่าการมองเห็นในการลงชื่อเข้าใช้สามารถเปิดเผยการคงอยู่ที่อิงตัวตนได้ การตรวจสอบเหล่านี้ตรวจไฮฟ์ SAM และ SOFTWARE ที่เก็บรวบรวมมา ไม่ใช่รายการบัญชี Active Directory การเป็นสมาชิกกลุ่มในโดเมน หรือการลงชื่อเข้าใช้ในอดีต

### การเป็นสมาชิกกลุ่ม Administrators {#admin_group_membership}

**Check ID:** `admin_group_membership` · **ATT&CK ตามแคตตาล็อก:** [T1098](https://attack.mitre.org/techniques/T1098/)

**สิ่งที่ Juntei ตรวจสอบ:** ถอดรหัสรายการสมาชิก alias ของ Administrators ภายในเครื่องจาก SAM และรายงาน SID ของเครื่องภายในที่ระบุบัญชีที่ถูกซ่อน หรือไม่ตรงกับบัญชี/กลุ่มใดใน SAM ที่เก็บรวบรวมมา

**หลักฐานที่ตรวจสอบ:**

- `SAM\Domains\Builtin\Aliases\00000220\C` และเรคคอร์ด SID ของเครื่อง/ผู้ใช้/กลุ่มภายใต้ `SAM\Domains\Account` ในไฮฟ์ SAM
- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList` สำหรับเชื่อมโยงชื่อที่ถูกซ่อน

**แนวทางการสืบสวน:** นี่คือการตรวจสอบความผิดปกติแบบเจาะจง ไม่ใช่รายชื่อผู้ดูแลระบบทั้งหมด SID ภายนอก/โดเมนจะไม่ถูกประเมินว่าเป็นผู้ใช้ภายในเครื่องที่ขาดหาย บัญชีที่ถูกลบ ช่วงเวลาที่เก็บรวบรวม หรือไฮฟ์ที่ไม่สมบูรณ์ต้องได้รับการสืบสวนก่อนสรุปว่าเป็นการเป็นสมาชิกที่เป็นอันตราย

### บัญชีภายในเครื่องที่ถูกซ่อน {#hidden_account}

**Check ID:** `hidden_account` · **ATT&CK ตามแคตตาล็อก:** [T1564.002](https://attack.mitre.org/techniques/T1564/002/)

**สิ่งที่ Juntei ตรวจสอบ:** รายงานค่าตัวเลขศูนย์ใน UserList ของ SpecialAccounts ใน Winlogon เพื่อระบุชื่อที่ถูกซ่อนจากหน้าจอลงชื่อเข้าใช้ปกติ

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Winlogon\SpecialAccounts\UserList`: ค่าที่เป็นชื่อบัญชี

**แนวทางการสืบสวน:** บัญชีบริการหรือบัญชีบำรุงรักษาอาจถูกซ่อนโดยเจตนา และค่านี้ไม่ได้ยืนยันว่าบัญชีที่ระบุชื่อมีอยู่หรือเป็นผู้ดูแลระบบ ให้เชื่อมโยงกับเรคคอร์ดบัญชีใน SAM และกิจกรรมการลงชื่อเข้าใช้

### RID Hijacking {#rid_hijacking}

**Check ID:** `rid_hijacking` · **ATT&CK ตามแคตตาล็อก:** [T1098](https://attack.mitre.org/techniques/T1098/)

**สิ่งที่ Juntei ตรวจสอบ:** เปรียบเทียบ RID เลขฐานสิบหกของซับคีย์บัญชีใน SAM กับ RID ที่จัดเก็บที่ offset 0x30 ในค่า F ของบัญชีนั้น และรายงานกรณีที่ไม่ตรงกัน

**หลักฐานที่ตรวจสอบ:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` ในไฮฟ์ SAM

**แนวทางการสืบสวน:** การไม่ตรงกันเป็นความผิดปกติร้ายแรงของเรคคอร์ดบัญชี แต่อาจต้องวิเคราะห์ความเสียหาย/การเก็บรวบรวมด้วย ให้ตรวจสอบไบต์ต้นทาง สถานะ transaction log และเรคคอร์ดบัญชี/การลงชื่อเข้าใช้ การตรวจสอบนี้ไม่ได้พิสูจน์ว่าผู้กระทำรายใดแก้ไขเรคคอร์ด

### RID Suborner (บัญชีผู้ดูแลระบบที่ถูกซ่อน) {#rid_suborner}

**Check ID:** `rid_suborner` · **ATT&CK ตามแคตตาล็อก:** [T1098](https://attack.mitre.org/techniques/T1098/)

**สิ่งที่ Juntei ตรวจสอบ:** รายงานบัญชีใน SAM ที่ RID ของซับคีย์ไม่ใช่ 500 แต่ RID ในค่า F เป็น 500 ซึ่งเป็นรูปแบบการแทนที่ตัวตนผู้ดูแลระบบที่เจาะจงกว่า

**หลักฐานที่ตรวจสอบ:**

- `SAM\Domains\Account\Users\<hexadecimal RID>\F` ในไฮฟ์ SAM

**แนวทางการสืบสวน:** การตรวจสอบนี้ซ้อนทับกับการตรวจสอบ RID ไม่ตรงกันทั่วไป แม้ชื่อในแคตตาล็อกจะระบุเช่นนั้น แต่การตรวจสอบนี้ไม่ได้กำหนดให้ต้องมีการตั้งค่าซ่อนการลงชื่อเข้าใช้แยกต่างหาก ให้ยืนยันการมองเห็นบัญชี สิทธิ์ และการใช้งานบัญชีจริงแยกกัน

## ส่วนประกอบการยืนยันตัวตนและการป้องกันข้อมูลประจำตัว { #authentication-components-and-credential-protection }

Local Security Authority (LSA) จัดการฟังก์ชันด้านความปลอดภัยผ่าน LSASS และคอมโพเนนต์ที่เกี่ยวข้อง authentication package, password filter และ network provider อาจเป็นซอฟต์แวร์ความปลอดภัยที่ถูกต้อง หรือเป็นช่องทางในการเข้าถึงข้อมูลประจำตัวอย่างต่อเนื่อง การตรวจสอบสองรายการรวบรวมรายการการตั้งค่าการป้องกัน แทนที่จะเป็นการเริ่มอัตโนมัติของไฟล์ปฏิบัติการ ให้ตีความโดยใช้บทบาทของโฮสต์ เวอร์ชันของ Windows และนโยบายที่มีผลจริง

### AMSI Provider DLL {#amsi_providers}

**Check ID:** `amsi_providers` · **ATT&CK ตามแคตตาล็อก:** [T1546.015](https://attack.mitre.org/techniques/T1546/015/)

**สิ่งที่ Juntei ตรวจสอบ:** แจกแจง CLSID ของ AMSI provider ระดับเครื่อง และระบุไลบรารี InprocServer32 แบบดั้งเดิม/32 บิตเมื่อมีข้อมูล การลงทะเบียนที่ระบุไม่ได้จะเก็บ class ID ไว้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SOFTWARE\Microsoft\AMSI\Providers\<CLSID>` และการลงทะเบียน InprocServer32 ของ `Classes\CLSID` / `Classes\Wow6432Node\CLSID` ที่ระบุได้

**แนวทางการสืบสวน:** แอนติไวรัสและผลิตภัณฑ์ความปลอดภัยลงทะเบียน AMSI provider อย่างถูกต้องตามปกติ ให้ตรวจทานที่มาของ provider/ไลบรารีและการโหลดจริง การลงทะเบียนไม่ได้พิสูจน์ว่ามีการหลบเลี่ยง AMSI หรือ provider เป็นอันตราย

### Authentication Packages {#authentication_packages}

**Check ID:** `authentication_packages` · **ATT&CK ตามแคตตาล็อก:** [T1547.002](https://attack.mitre.org/techniques/T1547/002/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `Authentication Packages` ของ LSA ที่ไม่ว่างแต่ละรายการจาก control set ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Authentication Packages` |

**แนวทางการสืบสวน:** เปรียบเทียบกับ Windows build ต้นทางและซอฟต์แวร์ยืนยันตัวตนที่ติดตั้ง แพ็กเกจที่ไม่ใช่ค่าเริ่มต้นอาจถูกต้องตามปกติ ให้ระบุ DLL และยืนยันกิจกรรมการโหลด/ข้อมูลประจำตัวก่อนสรุปว่ามีการดักจับ

### พฤติกรรมการลงชื่อเข้าใช้ของผู้ดูแลระบบ DSRM {#dsrm_backdoor}

**Check ID:** `dsrm_backdoor` · **ATT&CK ตามแคตตาล็อก:** [T1547.001](https://attack.mitre.org/techniques/T1547/001/)

**สิ่งที่ Juntei ตรวจสอบ:** รายงานค่าตัวเลข `DsrmAdminLogonBehavior=2` ในตำแหน่ง ControlSet001, ControlSet002 หรือ CurrentControlSet ที่มีการใช้งาน การตั้งค่านี้สามารถอนุญาตให้ใช้บัญชี DSRM นอกบริบทการกู้คืนตามปกติได้

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\ControlSet001\Control\Lsa\DsrmAdminLogonBehavior` รวมถึง ControlSet002 และ CurrentControlSet เมื่อมีอยู่

**แนวทางการสืบสวน:** เกี่ยวข้องกับ domain controller เป็นหลัก ให้ระบุว่า control set ใดใช้งานอยู่โดยใช้ Select ใน SYSTEM ผลที่ตรงกันใน control set ในอดีตไม่จำเป็นต้องมีผลจริง ค่านี้ไม่ได้พิสูจน์ว่าผู้โจมตีรู้รหัสผ่าน DSRM หรือได้ลงชื่อเข้าใช้

### การกำหนดค่า Credential Guard {#lsa_cfg_flags}

**Check ID:** `lsa_cfg_flags` · **ATT&CK ตามแคตตาล็อก:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `LsaCfgFlags` ของ LSA ที่กำหนดค่าไว้ รวมถึงค่าที่เปิดใช้งานตามปกติและค่าศูนย์ การตรวจสอบนี้รวบรวมรายการการกำหนดค่า Credential Guard แทนที่จะเป็นการเริ่มอัตโนมัติของไฟล์ปฏิบัติการ

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `LsaCfgFlags` |

**แนวทางการสืบสวน:** ตีความค่าร่วมกับเวอร์ชันของ Windows เงื่อนไขเบื้องต้นด้านเฟิร์มแวร์/ความปลอดภัย และนโยบายที่มีผลจริง ค่าศูนย์ที่จัดเก็บไว้หรือการไม่มีค่าเพียงอย่างเดียวไม่ได้ยืนยันการเปลี่ยนแปลงโดยผู้โจมตี สถานะการป้องกันที่มีผลจริง หรือการขโมยข้อมูลประจำตัว

### LSA Password Filter {#lsa_password_filter}

**Check ID:** `lsa_password_filter` · **ATT&CK ตามแคตตาล็อก:** [T1556.002](https://attack.mitre.org/techniques/T1556/002/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `Notification Packages` ของ LSA ชื่อไลบรารีที่ไม่มีพาธจะถูกระบุเทียบกับไดเรกทอรี Windows System32 ที่เก็บรวบรวมมา โดยเพิ่ม `.dll` เมื่อเหมาะสม

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Notification Packages` |

**แนวทางการสืบสวน:** ผลิตภัณฑ์นโยบายรหัสผ่านมักติดตั้ง filter ให้ตรวจสอบไลบรารีและหลักฐานการโหลด และแยก filter ที่กำหนดค่าไว้ออกจากหลักฐานว่ามีการเปลี่ยนรหัสผ่านหรือการดักจับข้อมูลประจำตัวเกิดขึ้น

### สถานะการป้องกัน PPL ของ LSASS {#lsa_run_as_ppl}

**Check ID:** `lsa_run_as_ppl` · **ATT&CK ตามแคตตาล็อก:** [T1547.008](https://attack.mitre.org/techniques/T1547/008/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านค่า `RunAsPPL` ของ LSA ที่กำหนดค่าไว้ รวมถึงค่าที่ไม่เป็นศูนย์ นี่คือการรวบรวมรายการการตั้งค่าการป้องกัน ไม่ใช่การทดสอบว่า LSASS ทำงานโดยไม่มีการป้องกัน

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `RunAsPPL` |

**แนวทางการสืบสวน:** การป้องกัน LSA ที่มีผลจริงขึ้นอยู่กับเวอร์ชันของ Windows นโยบาย การล็อกด้วยเฟิร์มแวร์ และสถานะการรีสตาร์ต การไม่มีแถวสำหรับค่าที่ไม่มีอยู่ไม่ใช่ผลการวัดการป้องกัน ให้ใช้หลักฐานอิสระจากระบบ/ขณะทำงานเพื่อประเมินการป้องกันที่มีผลจริง

### Network Provider DLL {#network_provider_dll}

**Check ID:** `network_provider_dll` · **ATT&CK ตามแคตตาล็อก:** [T1556.008](https://attack.mitre.org/techniques/T1556/008/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่าน `NetworkProvider\ProviderPath` ภายใต้บริการต่าง ๆ ใน control set ของ SYSTEM ที่ใช้งานอยู่

**หลักฐานที่ตรวจสอบ:**

- `HKLM\SYSTEM\<ControlSet>\Services\<service>\NetworkProvider\ProviderPath`

**แนวทางการสืบสวน:** network logon provider อาจถูกต้องตามปกติ ให้ตรวจสอบลำดับ/ความเกี่ยวข้องของ provider ซอฟต์แวร์เครือข่าย/ความปลอดภัยที่ติดตั้ง และการโหลดไลบรารีจริง ค่า ProviderPath เพียงอย่างเดียวไม่ได้พิสูจน์ว่ามีการดักจับข้อมูลประจำตัว

### Security Packages {#security_packages}

**Check ID:** `security_packages` · **ATT&CK ตามแคตตาล็อก:** [T1547.005](https://attack.mitre.org/techniques/T1547/005/)

**สิ่งที่ Juntei ตรวจสอบ:** อ่านรายการ `Security Packages` ของ LSA ที่ไม่ว่างแต่ละรายการ ซึ่งครอบคลุม security support provider ที่กำหนดค่าไว้

**หลักฐานที่ตรวจสอบ:**

| คีย์รีจิสทรี | ค่าที่ตรวจสอบ |
|---|---|
| `HKLM\SYSTEM\<ControlSet>\Control\Lsa` | `Security Packages` |

**แนวทางการสืบสวน:** เปรียบเทียบกับ baseline ของ Windows/ผลิตภัณฑ์ความปลอดภัยที่คาดว่าจะมี และตรวจสอบแต่ละไลบรารี การกำหนดค่าไม่ได้พิสูจน์ว่า provider ถูกโหลด ดักจับข้อมูลประจำตัว หรือถูกเพิ่มระหว่างเหตุการณ์

## จากผู้ต้องสงสัยสู่ข้อสรุป { #work-from-a-candidate-to-a-conclusion }

1. เปิดแหล่งข้อมูลต้นฉบับที่ระบุโดย `SourceFile` ยืนยันค่าฉบับเต็ม action ใน XML
   สคริปต์ หรือเรคคอร์ดฐานข้อมูลที่กู้คืนมา และผู้ใช้/โฮสต์ที่แน่ชัดซึ่งเป็นเจ้าของ
2. พิจารณาว่ากลไกนั้นสามารถทำงานได้หรือไม่: ตรวจสอบแฟล็กการเปิดใช้งาน เวอร์ชันของ runtime หรือ Windows
   สถานะของบริการ/งาน ทริกเกอร์ที่จำเป็น ผู้ใช้ สถาปัตยกรรม และ control set ที่ใช้งานอยู่
3. ตรวจสอบโปรแกรม DLL สคริปต์ หรือ assembly ที่ถูกเลือก คำนวณแฮชของไบต์ที่เก็บรวบรวมมา และตรวจทานอาร์กิวเมนต์
   และเนื้อหา launcher ของ Windows ที่จดจำได้หรือสตริงผู้เผยแพร่ไม่ได้ชี้ขาดความถูกต้องตามกฎหมาย
4. ยืนยันสิ่งที่เกิดขึ้นจริงโดยใช้หลักฐานอิสระ ตัวอย่างเช่น การสร้างโพรเซส
   การโหลด image ของ Sysmon เรคคอร์ดขณะทำงานของ Task Scheduler กิจกรรมของบริการ Prefetch และไฟล์ที่เกี่ยวข้อง
   เหตุการณ์การเปลี่ยนแปลงการกำหนดค่าสามารถพิสูจน์ว่าการตั้งค่าเปลี่ยนไป โดยไม่ได้พิสูจน์ว่า payload ได้ทำงาน
5. สร้างไทม์ไลน์โดยใช้ timestamp จากแหล่งข้อมูลต้นฉบับและที่มาของการเชื่อมโยงที่บันทึกไว้
   แยกการเขียนคีย์ การเปลี่ยนแปลงการติดตั้ง การเขียนไฟล์/คอนเทนเนอร์ และการเรียกใช้งาน ออกเป็นข้อเท็จจริงแยกกัน
6. บันทึกข้อจำกัดควบคู่ไปกับผลการตรวจพบ: ไฮฟ์ที่ขาดหาย ไฟล์เป้าหมายที่ขาดหาย principal ของงานที่ไม่ทราบ
   เมทาดาทาผู้ลงนามที่ไม่ได้ตรวจสอบ เรคคอร์ดในอดีตที่ได้จากการ carve หรือทริกเกอร์ที่ไม่สามารถยืนยันได้

สำหรับไวยากรณ์คำสั่ง โปรไฟล์ ฟิลด์เอาต์พุตทุกรายการ และสถานะ coverage ให้กลับไปที่
[คู่มือคำสั่ง Persistence Hunter](../commands/persistence-hunter.md) ส่วน
[คู่มืออาร์ติแฟกต์](../artifacts/index.md) อธิบายแหล่งข้อมูลพื้นฐาน และ
[กิตติกรรมประกาศ](acknowledgements.md) ระบุเครื่องมือด้านการคงอยู่ต้นทาง
