# ข้อมูลที่ Microsoft Defender กักกัน { #microsoft-defender-quarantine }

## สิ่งที่อยู่ในพื้นที่กักกัน { #what-the-quarantine-contains }

พื้นที่กักกัน (quarantine) ของ Microsoft Defender เก็บรักษาเมทาดาทาเกี่ยวกับการตรวจจับ และเนื้อหาที่เข้ารหัสของ
ทรัพยากรที่ถูกกักกัน Juntei เชื่อมโยงระเบียน Entries, ResourceData และ Resources เข้าด้วยกันเพื่อรายงาน
ชื่อภัยคุกคาม เวลาที่กักกัน พาธเชิงตรรกะและพาธเชิงกายภาพเดิม เมทาดาทาของไฟล์ เจ้าของ
`Zone.Identifier` ที่ระบุแหล่งดาวน์โหลด แฮช และปัญหาด้านความสมบูรณ์ของข้อมูล

อาร์ติแฟกต์นี้อาจคงอยู่นานกว่า event log ที่บันทึกการตรวจจับ/การแก้ไขไว้แต่แรก และยัง
อาจเก็บรักษาไบต์ที่น่าสงสัยไว้ครบถ้วนเพื่อทำวิศวกรรมย้อนกลับในสภาพแวดล้อมที่แยกออกมาได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Defender ได้สร้างระเบียนการกักกันสำหรับ
ภัยคุกคาม/ทรัพยากรที่ระบุชื่อ ณ เวลาที่บันทึกไว้ ป้ายกำกับของผลิตภัณฑ์ไม่ใช่ข้อพิสูจน์อิสระว่าเป็นมัลแวร์
และการกักกันเพียงอย่างเดียวไม่ได้พิสูจน์ว่าเคยมีการเรียกใช้งานมาก่อน

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Quarantine\
  Entries\{GUID}
  ResourceData\<prefix>\<SHA-1>
  Resources\<prefix>\<SHA-1>
```

ให้เก็บรวบรวมไดเรกทอรี `Quarantine` ทั้งหมด entry ที่ไม่มีไฟล์เนื้อหา/ระเบียนที่ตรงกันยังคงเป็น
เมทาดาทาที่มีประโยชน์ แต่ไม่สามารถให้ไบต์ของไฟล์หรือแฮชที่คำนวณได้

## ความปลอดภัยและข้อควรระวังสำหรับนักวิเคราะห์ { #safety-and-analyst-cautions }

- เนื้อหาที่ดึงออกมามีแนวโน้มเป็นอันตราย ให้ใช้ระบบวิเคราะห์ที่แยกออกมาและห้ามเรียกใช้งานเด็ดขาด
- `ThreatName` คือ signature/การจำแนกประเภทของ Defender ณ เวลานั้น อาจเป็นผลบวกลวง (false positive) หรือ
  ถูกจัดประเภทใหม่ในภายหลังได้
- เวลาที่กักกันคือเวลาแก้ไข/กักกัน ไม่จำเป็นต้องเป็นเวลาดาวน์โหลด การสร้าง หรือการเรียกใช้งาน
- ไฟล์ที่ถูกกักกันอาจถูกตรวจจับก่อนการเรียกใช้งานหรือหลังจากที่ทำงานไปแล้วก็ได้
- พาธเดิมและ Zone.Identifier มีคุณค่า แต่อาจไม่มีอยู่หรือถูกผู้โจมตีควบคุม
- ระเบียนกำพร้า (orphan) อาจเกิดจากการเก็บรักษา/การล้างข้อมูล การเก็บรวบรวมที่ไม่ครบถ้วน หรือความเสียหาย
- Juntei ตรวจสอบขนาด CRC, ID และแฮช ให้ตรวจทาน `Problems` ก่อนเชื่อถือไบต์ที่กู้คืนได้

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-quarantine <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find quarantine folders
  -f, --file <FILE>      Parse one Entries\{GUID} file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
      --extract <DIR>       Extract dangerous content with a .quarantined extension
  -C, --clobber             Overwrite confirmed output targets
```

```bash
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv
juntei parse-file defender-quarantine -d evidence/ -t jsonl \
  | jq '{QuarantineTime, ThreatName, Path, SHA256: .Content.SHA256, Problems}'

# Only in an isolated malware-analysis environment
juntei parse-file defender-quarantine -d evidence/ -o quarantine.csv --extract extracted/
```

ตัวอย่างเช่น `tool.exe` จะกลายเป็น `tool.exe.quarantined` สตรีมเสริมจะมีชนิดของสตรีมอยู่ใน
ชื่อ เช่น `tool.exe.Zone.Identifier.quarantined` นอกจากนี้ Juntei ยังลบสิทธิ์การเรียกใช้งานออก
นามสกุลไฟล์และสิทธิ์ช่วยลดการเรียกใช้งานโดยไม่ตั้งใจ แต่ไบต์ที่กู้คืนได้ยังคงต้อง
ถือว่าเป็นมัลแวร์

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Record` | `resource`, entry ที่ไม่มีทรัพยากร, `orphan-content` หรือ `orphan-record` ระเบียนกำพร้าแสดงให้เห็นการเชื่อมโยง/การเก็บรวบรวมที่ไม่ครบถ้วน |
| `QuarantineTime` | เวลากักกัน/แก้ไขของ Defender เป็น UTC ไม่ใช่เวลาการเรียกใช้งาน |
| `ThreatName`, `ThreatID` | การจำแนกประเภทของ Defender และ ID ที่เป็นตัวเลข ให้ถือเป็นคำตัดสิน/บริบทของผู้ผลิต |
| `ResourceType` | หมวดหมู่ทรัพยากร เช่น ไฟล์ คีย์รีจิสทรี บริการ รายการเริ่มต้นระบบ งานที่ตั้งเวลาไว้ หรืออื่น ๆ |
| `Path`, `PhysicalPath` | ตำแหน่งเชิงตรรกะ/เชิงกายภาพเดิม เป็นจุดเชื่อมโยงหลัก ให้เก็บรักษาการสะกดไว้ตรงตามเดิม |
| `FileSize` | ขนาดที่บันทึกในเมทาดาทาของทรัพยากร ให้เปรียบเทียบกับ `ContentSize` และ MFT |
| `CreationTime`, `LastAccessTime`, `LastWriteTime` | เมทาดาทาที่บันทึกไว้ของทรัพยากรที่ถูกกักกัน แยกจากเวลาที่กักกัน และอาจถูก timestomping |
| `Attr` | แอตทริบิวต์ของไฟล์ที่บันทึกไว้ สถานะ hidden/system อาจเป็นบริบทที่มีประโยชน์ |
| `Owner` | SID จาก security descriptor ที่กู้คืนได้ ความเป็นเจ้าของไม่จำเป็นต้องหมายถึงผู้ใช้ที่เรียกใช้งาน |
| `ZoneID`, `ReferrerURL`, `HostURL` | โซนการดาวน์โหลดและค่า URL จาก `Zone.Identifier` เป็นเบาะแสแหล่งที่มาที่แข็งแกร่งเมื่อมีอยู่ แต่ ADS อาจไม่มีอยู่/ถูกลบไปแล้ว |
| `ContentSize` | ขนาดจริงของเนื้อหาหลักที่กู้คืนได้ ให้เปรียบเทียบกับ `FileSize` ที่บันทึกไว้ |
| `MD5`, `SHA1`, `SHA256` | คำนวณจากไบต์ที่กู้คืนได้ ใช้ SHA-256 สำหรับการระบุเนื้อหาในปัจจุบัน และเก็บทุกค่าไว้เพื่อการทำงานร่วมกับระบบอื่น |
| `Streams` | ชนิด/ชื่อ/ขนาดของสตรีมที่กู้คืนได้ อาจรวมถึง ADS ข้อมูลความปลอดภัย หรือไฮฟ์รีจิสทรี |
| `ResourceID` | ตัวระบุที่เชื่อมโยงระเบียนเมทาดาทา/เนื้อหาของทรัพยากร |
| `EntryID` | GUID ของ entry การกักกัน ใช้เชื่อมโยงกับประวัติของ Defender/หลักฐานจากเหตุการณ์ได้ด้วย |
| `ScanID` | ตัวระบุการสแกนของ Defender สำหรับจัดกลุ่มทรัพยากร/การดำเนินการ |
| `QuarantineFolder` | โฟลเดอร์ต้นทางระดับราก ใช้แยกแยะโฮสต์ |
| `EntryFile`, `ContentFile`, `ResourceRecordFile` | ไฟล์ต้นทางที่แน่นอนที่ใช้สร้างแถวนี้ จำเป็นสำหรับการตรวจสอบความถูกต้อง |
| `ExtractedFile` | ไฟล์หลักที่ถูกกักกันซึ่ง `--extract` เขียนออกมา จะว่างเปล่าเมื่อไม่มีการดึงไฟล์หลักออกมา สตรีมเสริมจะถูกเขียนไว้ข้างไฟล์นี้ `auto-parse -x` ระบุพาธนี้โดยอิงจากไดเรกทอรีเอาต์พุต (`./HOST1/SecurityProducts/DefenderQuarantined/…`) |
| `Problems` | ความสมบูรณ์ไม่ตรงกัน เนื้อหาขาดหาย ID ไม่สอดคล้องกัน ความเสียหาย หรือบริบทของระเบียนกำพร้า ห้ามซ่อนไว้ |

JSON/JSONL จะเก็บรักษาทรัพยากรแบบมีโครงสร้าง รายการสตรีม entry ของระเบียน และรายละเอียดความสัมพันธ์
ทั้งหมด แทนที่จะเป็นข้อความที่ต่อกันเหมือนใน CSV

## พฤติกรรมการดึงข้อมูลออกมา { #extraction-behavior }

Juntei ใช้ชื่อไฟล์เดิมต่อท้ายด้วยนามสกุล `.quarantined` เมื่อ entry การกักกัน
บันทึกชื่อไฟล์ไว้ และใช้ตัวระบุเนื้อหาสำหรับทรัพยากรกำพร้าหรือทรัพยากรที่ไม่ใช่ไฟล์ พร้อมเพิ่ม
ตัวระบุเพื่อแก้ปัญหาชื่อไฟล์ซ้ำกัน อักขระที่ใช้ในชื่อไฟล์บน Windows ไม่ได้จะถูกแทนที่ สตรีม
ความปลอดภัยและสตรีมทางเลือก (alternate stream) จะถูกเขียนไว้ข้างไฟล์หลัก สตรีมความปลอดภัยแต่ละรายการจะมี
รายงานแบบข้อความที่อ่านได้ด้วย Juntei ลบสิทธิ์การเรียกใช้งานออก
และจะไม่เขียนทับเนื้อหาที่มีอยู่แล้วซึ่งแตกต่างกันหากไม่ระบุ `-C` การควบคุมเหล่านี้ช่วยลดอุบัติเหตุ แต่
ไม่ได้ทำให้ไบต์เหล่านั้นปลอดภัย

### การอ่าน security descriptor ที่กู้คืนได้ { #reading-the-recovered-security-descriptor }

ไฟล์ `.security.quarantined` มีเมทาดาทาความปลอดภัยของ Windows แบบไบนารี จึงไม่สามารถอ่านได้ใน
โปรแกรมแก้ไขข้อความ ไฟล์นี้เป็นสตรีมสำรองที่แยกต่างหาก ไม่ใช่สำเนาอีกชุดของไฟล์ปฏิบัติการ Juntei จะ
ถอดรหัสไฟล์นี้โดยอัตโนมัติเมื่อดึงข้อมูลออกมา ไม่ว่าจะด้วย `parse-file defender-quarantine --extract` หรือ `auto-parse -x`:

```text
tool.exe.quarantined              Original file content; treat as malware
tool.exe.security.quarantined     Original binary security descriptor, preserved unchanged
tool.exe.security_descriptor.txt  Readable ownership and permission report
```

ชื่อไฟล์ข้อความจะตามชื่อไฟล์ไบนารีจริง รวมถึงตัวระบุที่ใช้แก้ชื่อซ้ำ (ถ้ามี) ข้อความที่สร้างขึ้น
เป็น UTF-8 ทั่วไป และใช้การป้องกันการเขียนทับแบบเดียวกับไฟล์อื่นที่ดึงออกมา โดยมี
ชื่อไฟล์ไบนารี จำนวนไบต์ และ SHA-256 เพื่อให้จับคู่รายงานกับต้นทางได้

| ฟิลด์ในรายงาน | วิธีตีความ |
|---|---|
| เจ้าของและกลุ่มหลัก | SID ที่บันทึกไว้ใน descriptor SID ที่รู้จักกันทั่วไปจะได้รับชื่อ เช่น `BUILTIN\Administrators` หรือ `NT AUTHORITY\SYSTEM` ส่วน SID อื่นจะไม่ถูกแปลงชื่อ RID ที่คุ้นเคยเพียงอย่างเดียวไม่เพียงพอที่จะระบุชื่อบัญชีภายในเครื่อง/โดเมน ความเป็นเจ้าของไม่ได้พิสูจน์ว่าใครสร้างหรือเรียกใช้ไฟล์ |
| แฟล็กควบคุม | แฟล็กตัวเลขต้นฉบับและชื่อของแฟล็ก รวมถึงการมีอยู่ของ ACL การสืบทอด และการป้องกัน |
| DACL | รายการควบคุมการเข้าถึง (ACE) ตามลำดับ: ชนิด allow/deny, SID ของบัญชี, permission mask, สิทธิ์ และแฟล็กการสืบทอด สิ่งเหล่านี้เป็นรายการที่บันทึกไว้ ไม่ใช่การคำนวณสิทธิ์การเข้าถึงที่มีผลจริงสำหรับ token ของผู้ใช้ |
| SACL | รายการ audit หรือ mandatory-integrity ที่บันทึกไว้ แฟล็ก audit-success/failure อธิบายการกำหนดค่า ไม่ใช่ประวัติการเข้าถึง การไม่มี SACL ไม่ได้ยืนยันว่าการตรวจสอบ (auditing) ทั้งหมดถูกปิดใช้งาน |
| Object GUID | GUID ที่ไม่บังคับสำหรับ ACE เฉพาะออบเจ็กต์ โดยสิทธิ์เฉพาะออบเจ็กต์จะคงไว้เป็นตัวเลข แทนที่จะติดป้ายผิดว่าเป็นสิทธิ์ของไฟล์ |
| SDDL | การแสดงเจ้าของ กลุ่ม และรายการ ACL ที่รองรับในรูปแบบ Security Descriptor Definition Language ให้ตรวจดูแฟล็กควบคุมแยกต่างหาก Juntei จะไม่ส่งออกสตริง SDDL บางส่วน หากมีองค์ประกอบใดเสียหายหรือไม่รองรับ |
| สถานะการถอดรหัสและปัญหา | `complete` หรือ `incomplete` พร้อมเหตุผล ACE ที่ไม่รองรับจะคง hex ดิบไว้ในรายงาน ไบนารีต้นฉบับจะถูกเก็บไว้เสมอเมื่อการดึงข้อมูลสำเร็จ ปัญหาการถอดรหัสจะปรากฏในฟิลด์ `Problems` ของระเบียนการกักกันด้วย |

DACL ที่ไม่มีอยู่ DACL แบบ NULL และ DACL ที่ว่างเปล่าจะถูกรายงานแยกกัน DACL ที่ไม่มีอยู่หรือเป็น NULL
ไม่มีการจำกัดการเข้าถึงแบบ discretionary ส่วน DACL ที่ว่างเปล่าไม่มี ACE ใดที่อนุญาตการเข้าถึง การตรวจสอบสิทธิ์
การเข้าถึงอื่น ๆ ของ Windows และ privilege ยังคงมีผล ACE ที่สืบทอดมาและ ACE แบบ inherit-only จะยังถูกระบุแฟล็กไว้อย่างชัดเจน

ตัวอย่างเช่น entry ที่ให้สิทธิ์ควบคุมเต็มรูปแบบที่สืบทอดมาแก่ Administrators อาจปรากฏดังนี้:

```text
Entry 1: ALLOW (type 0x00)
  Flags: 0x10 (INHERITED_ACE)
  Account: S-1-5-32-544 (BUILTIN\Administrators)
  Access mask: 0x001F01FF
  Rights: Full control (FILE_ALL_ACCESS)
```

ตัวถอดรหัสทำงานแบบออฟไลน์บน Windows, macOS และ Linux โดยไม่ใช้ Windows API, Python หรือการค้นหาบัญชี
โดยถอดรหัส ACE พื้นฐานแบบ allow/deny/audit/alarm, object ACE และป้าย mandatory-integrity ส่วน ACE แบบ conditional,
callback, resource-attribute และ ACE อื่นที่ไม่รองรับจะถูกระบุอย่างชัดเจนและเก็บรักษาไว้ใน
รูปแบบดิบ โดยไม่มีการประเมินเงื่อนไขการเข้าถึงของ ACE เหล่านั้น descriptor ที่เสียหายจะไม่หยุดการดึงข้อมูล
ของสตรีมอื่น

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เริ่มจาก `ThreatName`, `Path`, `QuarantineTime`, `EntryId` และแฮช
2. ตรวจสอบ `Problems` และเปรียบเทียบขนาดที่บันทึกไว้กับเนื้อหาที่กู้คืนได้
3. ใช้ SHA-256 เชื่อมโยงไปยังโฮสต์อื่น EDR, sandbox และข้อมูล threat intelligence
4. เชื่อมโยงพาธ/เวลาเดิมกับ `$MFT`, บันทึก USN, Prefetch, Amcache และ event log
5. ใช้ฟิลด์ URL/โซนเพื่อตรวจสอบการเข้าถึงครั้งแรก (initial access) แต่ให้ยืนยันด้วยหลักฐานจากเบราว์เซอร์/พร็อกซี/DNS

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [ปลั๊กอิน Defender ของ dissect.target](https://docs.dissect.tools/en/latest/projects/dissect.target/plugins/os/windows/defender.html)
- [Microsoft: การแก้ไขและการกักกันของ Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/restore-quarantined-files-microsoft-defender-antivirus)
- [defender-dump](https://github.com/knez/defender-dump)
- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft: Security Descriptor String Format](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: ACE strings](https://learn.microsoft.com/en-us/windows/win32/secauthz/ace-strings)
