# รูปแบบเอาต์พุต { #output-formats }

## ข้อมูลสรุปในเทอร์มินัล { #terminal-summaries }

หากไม่ใช้ `-o` หรือ `-t` แบบมีโครงสร้าง โดยทั่วไปคำสั่งจะแสดงจำนวน ช่วงเวลา ความเสียหาย และข้อค้นพบที่มี
คุณค่าสูง ข้อมูลสรุปช่วยให้คุณเข้าใจขอบเขตก่อนสร้างเอาต์พุตขนาดใหญ่

## CSV { #csv }

CSV เป็นรูปแบบเริ่มต้นสำหรับการบันทึก สะดวกสำหรับเครื่องมือสเปรดชีต การนำเข้า SQL และโปรแกรมดู
ไทม์ไลน์ พาร์เซอร์บางตัวสร้างหลายตารางเนื่องจากอาร์ติแฟกต์มีเรคคอร์ดหลาย
ชนิด (Amcache, Jump Lists และ SRUM)

!!! warning "ให้ถือว่าคอลัมน์หลักฐานเป็นข้อความ"
    พาธหรือบรรทัดคำสั่งอาจขึ้นต้นด้วย `=`, `+`, `-` หรือ `@` ซอฟต์แวร์สเปรดชีตอาจตีความ
    เป็นสูตร ให้นำเข้าทุกฟิลด์เป็นข้อความหรือใช้โปรแกรมดู CSV ที่ปลอดภัยสำหรับงานนิติวิทยาศาสตร์ Juntei เก็บรักษา
    สตริงของหลักฐานไว้ตามเดิมแทนที่จะแก้ไข

## ชื่อฟิลด์ { #field-names }

ส่วนหัวคอลัมน์ CSV และชื่อฟิลด์ JSON/JSONL ใช้ PascalCase รวมถึง Amcache และ
manifest ของ `auto-parse` ตัวย่อยังคงเป็นตัวพิมพ์ใหญ่: `ProcessID`, `TimestampUTC`, `UserSID`,
`SHA256` คำที่ตามหลังตัวย่อจะคั่นด้วยยัติภังค์: `IP-Address`, `MFT-Entry`,
`USN-Journal` เมื่อใช้ `jq` ให้ใส่เครื่องหมายคำพูดรอบฟิลด์ที่มียัติภังค์: `.["IP-Address"]`

CSV ของ MFT แบบเริ่มต้นใช้หัวข้อที่สั้นและอ่านง่าย เช่น `EntryNum`, `Ext`, `Dir`, `HasADS`,
`Size`, `Created (SI)` และ `Created (FN)` ดู schema CSV ทั้งหมดได้ที่[ข้อมูลอ้างอิงฟิลด์ของ MFT](../artifacts/mft.md)
คู่ timestamp เรียงตามลำดับ MACB: DataModified, Accessed,
MetadataChanged, Created โดย SI มาก่อน FN ให้ใส่เครื่องหมายคำพูดรอบหัวข้อที่มีช่องว่างหรือเครื่องหมายวรรคตอน
ในเครื่องมือ CSV

ค่าของหลักฐานจะคงการสะกดดั้งเดิม คีย์ดิกชันนารีที่มาจากแหล่งข้อมูล เช่น ชื่อค่ารีจิสทรี
ภายใต้ `Details`/`DataRaw` และชื่อใน `Zone.Identifier` ภายใต้ `ZoneIdentifier` ก็จะ
คงเดิมเช่นกัน ดิกชันนารี `ValueState` และ `Raw` ของ SRUM ใช้ป้ายชื่อเดียวกับคอลัมน์ข้อมูล
ของตน การชนกันจากการ normalize จะถูกรายงาน แทนที่จะตัดฟิลด์ทิ้งโดยไม่แจ้ง

สิ่งนี้เปลี่ยน schema การส่งออกก่อนหน้า ให้สร้างผลลัพธ์ที่มีอยู่ใหม่และอัปเดตคิวรีที่บันทึกไว้
(เช่น `.record_type` กลายเป็น `.RecordType` และ `full_path` ของ Amcache กลายเป็น `FullPath`)
รายงานแบบข้อความที่มนุษย์อ่านได้และรูปแบบอินพุต/การกำหนดค่ายังคงใช้รูปแบบเดิม

## ชื่อฟิลด์แบบย่อ { #abbreviated-field-names }

ป้ายชื่อต่อไปนี้ใช้กับการส่งออก CSV, JSON และ JSONL รวมถึงการพาร์สอัตโนมัติและ
manifest ให้อัปเดตตัวกรอง `jq`, `xsv` และสเปรดชีตที่บันทึกไว้เมื่อสร้างผลลัพธ์เก่าใหม่
เปลี่ยนเฉพาะป้ายชื่อเอาต์พุตเท่านั้น ค่าของหลักฐานและชื่อค่ารีจิสทรีดิบจะคงการสะกดดั้งเดิม

| ป้ายชื่อเดิม | ป้ายชื่อปัจจุบัน |
|---|---|
| `ADS` | `HasADS` |
| `Application` | `App` |
| `ApplicationCompany` | `AppCompany` |
| `ApplicationCompanyRaw` | `AppCompanyRaw` |
| `ApplicationPath` | `AppPath` |
| `Arguments` | `Args` |
| `Attributes` | `Attr` |
| `AttributeType` | `AttrType` |
| `BytesRecvd` | `BytesReceived` |
| `Computer` | `Comp` |
| `ComputerName` | `CompName` |
| `ComputerNameLastWrittenUTC` | `CompNameLastWrittenUTC` |
| `Directories` | `Dir` |
| `FileName` | `Filename` |
| `MicrosecondZeros` | `ZeroU-Secs` |
| `SI-BeforeFN` | `SI<FN` |
| `UserName` | `User` |
| `Username` | `User` |
| `UserNameSource` | `UserSource` |
| `Users` | `User` |

เอาต์พุต MFT CSV ใช้ชื่อคอลัมน์เฉพาะ: `File` (Filename), `SN` (Sequence Number), `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number), `USN` (Update Sequence Number), `LogfileSN` (Logfile Sequence Number), `SID` (Security ID), `Type` (Name Type) และ `RefCnt` (Reference Count) ซึ่งใช้กับ MFT CSV จาก `auto-parse` ด้วย ดู [ฟิลด์เอาต์พุต MFT](../artifacts/mft.md#csv-output-fields)

เวลาใน JSON แบบเนทีฟของข้อมูลระบบคือ `CompNameLastWritten` (CSV:
`CompNameLastWrittenUTC`) `HasADS` เป็นตัวบ่งชี้แบบบูลีน `Dir` ใน Prefetch มีรายการ
ไดเรกทอรีที่บันทึกไว้ `User` คงชนิดเฉพาะของแต่ละอาร์ติแฟกต์: เป็นสตริงสำหรับชื่อบัญชี
หรือเป็นรายการผู้ใช้ที่เกี่ยวข้องในเอาต์พุต USB ไม่ได้หมายถึงตัวตนที่เรียกใช้งาน
ให้ใส่เครื่องหมายคำพูดรอบเครื่องหมายวรรคตอนในคิวรี: `."SI<FN"` และ `.["ZeroU-Secs"]`

## JSON { #json }

JSON จัดเก็บผลลัพธ์ทั้งหมดเป็นเอกสารเดียว โดยเก็บรักษาอาร์เรย์ โครงสร้างแบบซ้อน ค่า null
จำนวนเต็มขนาดใหญ่ และความแตกต่างที่ CSV แบบแบนไม่สามารถแสดงได้เสมอไป

```bash
juntei parse-file lnk -f sample.lnk -o lnk.json -t json
```

## JSON Lines { #json-lines }

JSONL เขียนอ็อบเจกต์ JSON ที่เป็นอิสระต่อกันหนึ่งอ็อบเจกต์ต่อบรรทัด เหมาะกับการสตรีมและทำงานร่วมกับ `jq` ได้อย่างเป็นธรรมชาติ:

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.StartType == "automatic") | {Host, ServiceName, ImagePath}'
```

เมื่อเอาต์พุตแบบมีโครงสร้างถูกส่งไปยัง standard output Juntei จะส่งความคืบหน้าและข้อความไปยัง standard
error เพื่อให้ไปป์ได้รับเฉพาะข้อมูล

`auto-parse -j timeline.jsonl` สร้างชุดข้อมูล JSONL เพิ่มเติมสำหรับชุดข้อมูลที่เก็บรวบรวมจากหลายโฮสต์
ทั้งหมด โดยยังคงผลลัพธ์ CSV ที่จัดหมวดหมู่ไว้ เพิ่ม `-t json` หรือ `-t jsonl` เพื่อเปลี่ยน
รูปแบบผลลัพธ์แต่ละรายการ แต่ละเรคคอร์ดมีฟิลด์ `Comp` และ `Artifact` เพื่อให้แหล่งที่มายังชัดเจน
หลังการนำเข้า SIEM

## UTC และความละเอียด { #utc-and-precision }

Juntei จัดรูปแบบเวลาที่พาร์สแล้วเป็น UTC โดยใช้ RFC 3339 เมื่อรูปแบบอนุญาต อาร์ติแฟกต์บางชนิดมี
ความละเอียด 100 นาโนวินาที บางชนิดจัดเก็บเป็นวินาที เวลา FAT ช่วงเวลาการรวมข้อมูล หรือไม่มีเวลาเลย
จำนวนหลักไม่ได้แปลว่าความแม่นยำ ให้ทำความเข้าใจฟิลด์ต้นทางที่อธิบายไว้ในหน้าอาร์ติแฟกต์ของฟิลด์นั้นเสมอ

UTC ใช้รูปแบบของ Windows event log `2026-10-20T08:46:38.2001668Z` โดยเก็บความละเอียด
ทศนิยมที่มีอยู่ไว้ `auto-parse -Z "+09:00"` แสดงช่วงเวลาที่พาร์สแล้วพร้อม offset แบบคงที่
ที่ระบุอย่างชัดเจนในทุกรูปแบบผลลัพธ์และการส่งออก `-j` (ไม่บังคับ) ค่าเริ่มต้นคือ UTC ไม่ว่า
คอมพิวเตอร์ที่เรียกใช้ Juntei จะเป็นเครื่องใด ค่าดิบและเวลานาฬิกาที่ไม่มีเขตเวลาจะคงไว้ตามที่บันทึก ดู
[เขตเวลาของเอาต์พุต](../commands/auto-parse.md#output-time-zone)

## ความเสียหายและค่าที่ขาดหาย { #damage-and-missing-values }

ค่าว่าง `null`, `NA`, `Problems` และฟิลด์สถานะเฉพาะของพาร์เซอร์ล้วนมีความหมาย ค่าเหล่านี้อาจหมายถึง
อาร์ติแฟกต์ไม่ได้จัดเก็บค่า เรคคอร์ดเสียหาย รูปแบบไม่รองรับ หรือ
พาร์เซอร์ไม่สามารถ resolve ค่าได้ อย่าแทนที่ค่าว่างทั้งหมดด้วยศูนย์
