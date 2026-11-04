# นโยบายการตรวจสอบ { #audit-policy }

## นโยบายการตรวจสอบคืออะไร { #what-it-is }

นโยบายการตรวจสอบ (audit policy) กำหนดว่า Windows จะบันทึกเหตุการณ์ความปลอดภัยใดลงใน Security event log
บน Windows Vista และใหม่กว่า นโยบายถูกกำหนดแยกตาม subcategory (59 รายการบน Windows 10 และ 11) และบน
Windows XP และ Server 2003 ถูกกำหนดแยกตาม category แต่ละรายการจะบันทึกความสำเร็จ ความล้มเหลว ทั้งสองอย่าง
หรือไม่บันทึกเลย

LSA เก็บนโยบายที่มีผลบังคับใช้ไว้ในไฮฟ์ `SECURITY` ไม่ว่านโยบายนั้นจะมาจาก `auditpol`, local
security policy หรือ Group Policy

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ระบบถูกตั้งค่าให้บันทึก (หรือไม่บันทึก)
เหตุการณ์เหล่านี้ ณ เวลาที่ไฮฟ์ถูกเขียนครั้งล่าสุด เมื่อ subcategory ไม่ได้ถูกตรวจสอบ การไม่มีเหตุการณ์
ไม่ได้บอกอะไรเลยว่ากิจกรรมนั้นเกิดขึ้นหรือไม่

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\PolAdtEv          (default value)
```

Windows Vista และใหม่กว่าเก็บส่วนหัวขนาด 12 ไบต์ ตามด้วยการตั้งค่าหนึ่งรายการต่อ subcategory เรียงทีละ category
แล้วตามด้วยจำนวน subcategory ของแต่ละ category เวอร์ชันต่าง ๆ แตกต่างกันเฉพาะจำนวนเหล่านั้น โดยแต่ละเวอร์ชัน
เพิ่ม subcategory ต่อท้าย category (56 รายการในไฮฟ์ Windows 8.x และ Server 2012 R2 ที่อ่าน
และ 59 รายการบน Windows 10, 11 และ Server 2022) Windows XP และ Server 2003 เก็บว่าเปิดการตรวจสอบหรือไม่
(หนึ่งไบต์ ตามด้วย padding ที่ Windows ไม่ล้าง) และการตั้งค่า category เก้ารายการ โครงสร้างนี้ได้รับการตรวจสอบกับ
อิมเมจ Windows XP สาธารณะของ Hacking Case ใน CFReDS ของ NIST ซึ่งไม่ได้ตรวจสอบสิ่งใดเลย ส่วนค่าของ
การตั้งค่า category เป็นไปตามเอกสารของ Microsoft

ให้เก็บรวบรวม `SECURITY` พร้อม transaction log และไฮฟ์ `SYSTEM` ของระบบ ซึ่งระบุชื่อ
โฮสต์

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **ข้อมูลที่ไม่รู้จักยังคงเป็นข้อมูลที่ไม่รู้จัก** revision ที่ไม่รองรับจะถูกเก็บรักษาไว้พร้อมข้อมูลวินิจฉัย แฟล็ก
  การตั้งค่าที่ไม่รู้จักจะทำให้ `Success` และ `Failure` ว่าง และจะไม่ถูกนับรวมในจำนวนการตั้งค่าที่ตรวจสอบ
  ตารางจำนวนที่เสียหายไม่สามารถเลื่อน word ที่สงวนไว้ให้กลายเป็นการตั้งค่าการตรวจสอบอื่นได้
  ข้อมูลที่เก็บมาอาจมีช่อง (slot) เพิ่มเติมเกินกว่าแค็ตตาล็อกชื่อ/GUID การตั้งค่าและ
  `SubcategoryId` ที่เป็นตัวเลขของช่องเหล่านั้นจะถูกเก็บไว้โดยมีชื่อ/GUID ว่าง ช่องที่จับคู่ไม่ได้ไม่ได้เป็น
  หลักฐานของความเสียหายในตัวเอง


- **ลำดับเป็นของ Windows เอง** ภายใน category หนึ่ง การตั้งค่าเรียงตามลำดับ message ID ของ
  subcategory ไม่ใช่ตาม GUID โดย Special Logon (`%%12548`, ดัชนี 4 ของ Logon/Logoff) อยู่ก่อน
  IPsec Quick Mode เครื่องมือที่อ่านตามลำดับ GUID จะรายงานค่าเริ่มต้นของ Windows 10 ว่า Special Logon เป็น
  "No Auditing" และ IPsec Quick Mode เป็น "Success" `SubcategoryId` เป็น ID เดียวกับที่ event 4719
  (System audit policy was changed) ให้ไว้ จึงจับคู่ทั้งสองได้
- **นโยบายที่มีผลบังคับใช้ ไม่ใช่แหล่งที่มา** Group Policy และ local policy เขียนค่าเดียวกัน
  `audit.csv` ของ Group Policy (ใน SYSVOL และ `C:\Windows\System32\GroupPolicy`) แสดงสิ่งที่
  ตั้งใจจะนำไปใช้ (`Machine\Microsoft\Windows NT\Audit\audit.csv` ของนโยบาย)
- **การเปลี่ยนแปลงตามเวลา:** `KeyLastWrittenUTC` บอกเวลาโดยประมาณที่นโยบายเปลี่ยนครั้งล่าสุด event 4719
  บันทึกการเปลี่ยนแปลงแต่ละครั้ง (ตราบเท่าที่ Security log ยังเก็บไว้) ผู้โจมตีสามารถปิดการตรวจสอบ
  แล้วเปิดกลับคืนได้
- **subcategory เทียบกับ category** บน Windows Vista และใหม่กว่า Windows ใช้ subcategory
  (เว้นแต่ตัวเลือกความปลอดภัย `SCENoApplyLegacyAuditPolicy` ถูกปิดและนโยบาย category แบบเดิม
  มีผลแทนที่)
- **Windows XP และ 2003:** เมื่อปิดการตรวจสอบ (`AuditingEnabled` เป็น False) จะไม่มี category ใดถูกตรวจสอบ
  ไม่ว่าจะตั้งค่าไว้อย่างไร
- **เหตุการณ์:** `Events` แสดงเหตุการณ์หลักของแต่ละ subcategory ตามเอกสารของ Microsoft
  เกี่ยวกับการตั้งค่า advanced audit policy ซึ่งเป็นเหตุการณ์ที่การตรวจสอบมักมองหา ไม่ใช่
  ทุกเหตุการณ์ที่ subcategory บันทึก

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry auditpol <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry auditpol -d collection/ -o auditpol.csv
juntei parse-registry auditpol -f SECURITY -t jsonl | jq -r 'select(.Setting == "none") | .Subcategory'
```

สำหรับแต่ละไฮฟ์ สรุปผลจะแสดงโครงสร้าง จำนวนการตั้งค่าที่ถูกตรวจสอบ และเวลาที่นโยบาย
ถูกเขียนครั้งล่าสุด จากนั้นแสดงรายการที่ตรวจสอบทั้งความสำเร็จและความล้มเหลว เฉพาะความสำเร็จ และเฉพาะ
ความล้มเหลว แล้วจึงแสดง subcategory ที่ไม่ได้ตรวจสอบพร้อมเหตุการณ์หลัก เหตุการณ์เหล่านั้นถูก
ปิดใช้งานในนโยบายที่เก็บมา แต่สิ่งนี้ไม่ได้ยืนยันนโยบายในช่วงที่กิจกรรมก่อนหน้าเกิดขึ้น

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | โฮสต์ (จากไฮฟ์ `SYSTEM` ของระบบ) และไฮฟ์ `SECURITY` |
| `Layout` | `subcategories` (Windows Vista และใหม่กว่า) หรือ `categories` (XP และ 2003) ว่างเมื่อถอดรหัสค่าไม่ได้ |
| `Category`, `Subcategory` | ตามชื่อที่ `auditpol` ใช้ |
| `SubcategoryGUID`, `SubcategoryID` | GUID ของ subcategory และ message ID ตามที่ event 4719 ให้ไว้ (`%%12548`) |
| `Setting` | `none`, `success`, `failure` หรือ `success_and_failure` |
| `Success`, `Failure` | บันทึกความสำเร็จและความล้มเหลวหรือไม่ |
| `AuditingEnabled` | Windows XP และ 2003: เปิดการตรวจสอบอยู่หรือไม่ |
| `Events` | Event ID หลักของ subcategory |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของ `PolAdtEv` คือเวลาโดยประมาณที่นโยบายเปลี่ยนครั้งล่าสุด |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสได้ไม่ครบทั้งหมด |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | โครงสร้างหรือขนาดที่ไม่รู้จัก จำนวนที่ไม่สอดคล้องกัน และค่าที่เกิน 3 |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ก่อนสรุปว่าบางสิ่งไม่ได้เกิดขึ้น ให้ตรวจสอบว่า subcategory ของสิ่งนั้นถูกตรวจสอบหรือไม่:
   Process Creation (4688), Logon (4624, 4625), Special Logon (4672), Other Object Access Events
   (scheduled task, 4698), Security System Extension (การติดตั้งบริการ, 4697), User Account และ
   Security Group Management (4720, 4732), File Share (5140), Kerberos (4768, 4769)
2. เปรียบเทียบนโยบายกับ Security log หากพบเหตุการณ์ของ subcategory ที่ปัจจุบันไม่ได้ตรวจสอบ
   แสดงว่านโยบายมีการเปลี่ยนแปลง ให้มองหา event 4719 และ `KeyLastWrittenUTC` ที่เป็นเวลาล่าสุด
3. เปรียบเทียบกับ `audit.csv` ของ Group Policy เพื่อแยกนโยบายระดับโดเมนออกจากการเปลี่ยนแปลงภายในเครื่อง
4. ใช้ `parse-registry read --recover-deleted` และ transaction log เพื่อหาเวอร์ชันก่อนหน้าของ
   `PolAdtEv`

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: การตั้งค่า advanced security audit policy](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/advanced-security-audit-policy-settings)
- [Microsoft: Auditing constants (GUID ของ subcategory)](https://learn.microsoft.com/en-us/windows/win32/secauthz/auditing-constants)
- [Microsoft: 4719(S) System audit policy was changed](https://learn.microsoft.com/en-us/previous-versions/windows/it-pro/windows-10/security/threat-protection/auditing/event-4719)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
