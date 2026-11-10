# บันทึกธุรกรรม NTFS (`$LogFile`) { #ntfs-transaction-log-logfile }

## คืออะไรและเหตุใดจึงสำคัญ { #what-it-is-and-why-it-matters }

NTFS คือระบบไฟล์ที่ Windows มักใช้กับวอลุ่มระบบ เพื่อให้กู้คืนได้หลังเกิดการล่มของระบบ NTFS จะ
บันทึกการเปลี่ยนแปลงที่ตั้งใจจะทำกับเมทาดาทาของระบบไฟล์ไว้ในไฟล์ซ่อนของวอลุ่มชื่อ
`$LogFile` การอัปเดตแต่ละครั้งอาจมีไบต์ **redo** (สถานะใหม่) และไบต์ **undo** (สถานะก่อนหน้า
ที่จำเป็นสำหรับการย้อนการเปลี่ยนแปลง) สิ่งนี้แยกจาก Windows Event Log ไฟล์ `.LOG1` ของรีจิสทรี
และ[บันทึก USN](usn-journal.md)

สำหรับผู้ตรวจสอบ สถานะก่อนหน้ามีประโยชน์เป็นพิเศษ ระเบียน undo อาจเก็บรักษาชื่อไฟล์ก่อนการเปลี่ยนชื่อ
directory entry ที่ถูกลบ หรือ timestamp ก่อนการอัปเดตเมทาดาทาไว้ได้
ระเบียนบางรายการยังมีไบต์ที่ NTFS เขียนลงในบันทึก USN รวมถึงระเบียนที่ไม่อยู่ใน
`$J` ที่เก็บรวบรวมมาแล้ว อย่างไรก็ตาม ธุรกรรมจำนวนมากเกี่ยวข้องกับกิจกรรมปกติของระบบปฏิบัติการ และไม่ได้
อธิบายการโจมตี

**ตัวอย่าง:** index entry ที่ถูกลบกล่าวถึง `report.txt` และ entry ที่เพิ่มเข้ามาภายหลังกล่าวถึง
`report.txt.encrypted` การจับคู่ file reference แบบเต็ม parent สายธุรกรรม และอาร์ติแฟกต์
อื่น ๆ สามารถสนับสนุนข้อสรุปว่ามีการเปลี่ยนชื่อ ชื่อทั้งสองเพียงลำพังไม่ได้ระบุ ransomware ผู้ควบคุม
หรือเวลาที่แน่นอนของการเข้ารหัส

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\$LogFile
D:\$LogFile
```

วอลุ่ม NTFS แต่ละวอลุ่มมี log ของตัวเอง ให้ได้มาด้วยเครื่องมือที่อ่านเมทาดาทาของ NTFS หรือส่งออก
จาก forensic image หรือ snapshot การคัดลอกไฟล์ตามปกติของ Windows อาจเข้าถึงไม่ได้ ให้เก็บรวบรวม
`$MFT` และ `$Extend\$UsnJrnl:$J` ของวอลุ่มเดียวกันในเวลาที่ใกล้เคียงกันที่สุด เก็บรักษาไฟล์
ต้นฉบับและแฮชไว้ การได้มาในภายหลังอาจมีช่วงกิจกรรมที่แตกต่างออกไป

`$LogFile` เป็น **log แบบวนรอบ (circular log)**: ระเบียนใหม่จะเขียนทับระเบียนเก่า ระยะเวลาที่เก็บข้อมูลขึ้นอยู่กับขนาดของ log
และปริมาณงาน และอาจเป็นเพียงไม่กี่นาทีหรือไม่กี่ชั่วโมง การไม่มีชื่อไฟล์หรือการดำเนินการใดไม่ใช่
หลักฐานว่าสิ่งนั้นไม่เคยเกิดขึ้น

## สิ่งที่ระเบียนพิสูจน์ได้ และข้อควรระวัง { #what-a-record-proves-and-cautions }

- การอัปเดตบันทึกเมทาดาทาที่ NTFS บันทึกไว้เพื่อการกู้คืน redo entry เพียงรายการเดียวไม่ได้พิสูจน์ว่า
  การกระทำระดับสูงทั้งหมดเสร็จสมบูรณ์ หรือว่าบุคคลใดบุคคลหนึ่งเป็นผู้เริ่ม ให้เชื่อมโยง
  ระเบียนที่เกี่ยวข้อง การเสร็จสิ้นของธุรกรรม ระบบไฟล์ที่เป็นผลลัพธ์ และหลักฐานอิสระ
- **ไม่มี timestamp ของธุรกรรมโดยทั่วไป** timestamp ที่ฝังอยู่อธิบายเมทาดาทาของไฟล์ ซึ่งบางครั้ง
  ล้าสมัย ถูกคัดลอกมา หรือถูกผู้โจมตีแก้ไข อย่านำ `CreatedOn` ไปใส่ในไทม์ไลน์ของเหตุการณ์
  ในฐานะเวลาของการเปลี่ยนชื่อหรือการเปลี่ยน timestamp ระเบียน USN อาจให้ timestamp แยกต่างหากได้
- LSN (log sequence number) ใช้เรียงลำดับระเบียน **ภายใน log ของวอลุ่มเดียว** ไม่ใช่เวลาตามนาฬิกา
  และต้องไม่นำไปเปรียบเทียบข้ามโฮสต์หรือข้ามวอลุ่ม ระเบียนจากรอบก่อนหน้าของ circular buffer
  อาจยังหลงเหลืออยู่ปะปนกับกิจกรรมที่ใหม่กว่า
- ช่อง (slot) ใน transaction table ถูกนำกลับมาใช้ใหม่ ให้จัดกลุ่มด้วย `TransactionFirstLsn`, `TransactionId`, ตัวตนของ
  client และแหล่งที่มา แทนที่จะใช้ `TransactionId` เพียงอย่างเดียว สายโซ่อาจชี้ออกไปนอกข้อมูลที่เก็บไว้
- file reference ประกอบด้วยหมายเลข entry **และหมายเลข sequence** มิฉะนั้นการนำ MFT entry กลับมาใช้ใหม่อาจ
  ทำให้ไฟล์ที่ไม่เกี่ยวข้องกันดูเหมือนเป็นไฟล์เดียวกัน
- พาธที่ได้จาก `$MFT` อธิบาย **สถานะของ MFT ที่เก็บรวบรวมมา** ไม่ใช่พาธในอดีตที่สร้างขึ้นใหม่
  Juntei จับคู่หมายเลข sequence และแสดงส่วนประกอบที่ไม่ทราบไว้อย่างชัดเจน parent อาจ
  ถูกเปลี่ยนชื่อหลังจากธุรกรรม และ hard link อาจให้ชื่อปัจจุบันได้มากกว่าหนึ่งชื่อ
- `TargetInferred` หมายความว่าการแมป open attribute มาจากช่วงอื่นของ log
  ให้ระมัดระวังเป็นพิเศษกับการแมปนั้น index entry ที่ไม่สามารถระบุได้อาจถูกถอดรหัสจาก
  โครงสร้างเพียงอย่างเดียว ให้ตรวจสอบชื่อด้วยไบต์ดิบและแหล่งข้อมูลอื่น
- หน้าที่ฉีกขาด (torn page) จะถูกข้าม ส่วนระเบียนรอบข้างที่อ่านได้ยังคงใช้งานได้ `Truncated` ระบุ
  ระเบียนที่ส่วนต่อเนื่องขาดหายไป ให้ตรวจทานข้อความวินิจฉัยของพาร์เซอร์ก่อนถือว่าเอาต์พุต
  ครบถ้วน อินพุตที่มีความยาวเป็นศูนย์ถือเป็นข้อผิดพลาด ส่วน log ที่ยังไม่ถูกเขียนซึ่งเป็น `FF` ทั้งหมดจะไม่มีแถวใดตามปกติ
- พาร์เซอร์ **ไม่ได้เล่นซ้ำ (replay)** ธุรกรรมเพื่อสร้าง MFT ในอดีตขึ้นใหม่ ไม่ได้
  กู้คืนเนื้อหาไฟล์แบบ non-resident ใด ๆ และไม่ได้เรียกใช้ข้อมูลใด ๆ ที่เก็บรวบรวมมา ไม่รองรับหน้าที่ป้องกันด้วย
  DAX CRC การตรวจสอบความถูกต้องกับหลักฐานจริงครอบคลุมหน้า log ขนาด 4 KiB เป็นหลัก

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```bash
# Single log, current paths from an adjacent $MFT when present
juntei parse-ntfs logfile -f '$LogFile' -o logfile.csv

# Explicit MFT from the SAME volume
juntei parse-ntfs logfile -f '$LogFile' -m '$MFT' -t jsonl -o logfile.jsonl

# Multiple collected logs; SourceFile distinguishes every input
juntei parse-ntfs logfile -d evidence -o collected-logs.csv

# Preserve the underlying payloads for low-level validation
juntei parse-ntfs logfile -f '$LogFile' --raw-data -t jsonl -o logfile-raw.jsonl

# Automatic per-computer output
juntei auto-parse -d evidence -o parsed
```

ดู[เอกสารอ้างอิงคำสั่ง](../commands/filesystem.md#parse-ntfs-logfile) สำหรับตัวเลือกทั้งหมด การค้นหา
ในไดเรกทอรียอมรับ `$LogFile`, `LogFile`, รูปแบบ `.bin` ของทั้งสอง และชื่ออย่าง `C_$LogFile` หรือ
`C_$LogFile.bin` โดยไม่สนใจตัวพิมพ์เล็กใหญ่ ไฟล์เดี่ยวที่ส่งผ่าน `-f` จะมีชื่ออะไรก็ได้

การจับคู่ MFT อัตโนมัติคำนึงถึง prefix ของวอลุ่มในข้อมูลที่ส่งออก: `C_$LogFile` ใช้ `C_$MFT` และ
`D_$LogFile.bin` จะลอง `D_$MFT.bin` ก่อน แล้วจึงลอง `D_$MFT` log ที่มี prefix จะไม่ใช้ `$MFT` ธรรมดา
เป็นทางเลือกสำรองเลย `LogFile` ที่ไม่มี prefix ยังยอมรับ `MFT` หรือ `$MFT` ด้วย ให้ใช้ `-m` ร่วมกับ `-f` เพื่อเลือก
MFT ที่ตรงกันอย่างชัดเจนเมื่อข้อมูลที่ส่งออกใช้รูปแบบการตั้งชื่ออื่น

เอาต์พุตเป็นหนึ่งระเบียนต่อหนึ่ง LSN รวมถึงระเบียน client แบบ update, checkpoint และแบบที่ไม่ทราบ JSON เป็น
สตรีมของออบเจ็กต์ที่จัดรูปแบบสวยงาม (pretty-printed) ส่วน JSONL มีหนึ่งออบเจ็กต์ต่อบรรทัด CSV มีชื่อฟิลด์เหมือนกัน
อาร์เรย์และฟิลด์แบบมีโครงสร้างจะเป็น JSON แบบกะทัดรัดภายในเซลล์ CSV ค่าสเกลาร์ที่ไม่ทราบจะว่างเปล่าใน
CSV และเป็น `null` ใน JSON อาร์เรย์ว่างหมายถึงไม่มีรายการใดถูกถอดรหัส ไม่ใช่ข้อพิสูจน์ว่าไม่มีรายการอยู่เลย

log ทั้งไฟล์และระเบียนที่วิเคราะห์แล้วจะถูกเก็บไว้ในหน่วยความจำครั้งละหนึ่งไฟล์ การระบุพาธจาก MFT จะเพิ่ม
การใช้หน่วยความจำและปริมาณงาน `--raw-data` อาจเพิ่มขนาดเอาต์พุตได้มาก ใน `auto-parse` ให้ใช้ `-T` เพื่อ
จำกัดจำนวนโพรเซสพาร์เซอร์ที่ทำงานพร้อมกันบนเครื่องที่มีหน่วยความจำจำกัด

หากอ่าน MFT ที่อยู่ข้างเคียงไม่ได้ Juntei จะแสดงคำเตือนและส่งออก log โดยไม่มีพาธ ซึ่ง `MftSource`
จะยังคงเป็น null ความล้มเหลวในการเสริมข้อมูลนี้จะไม่ทิ้งหลักฐานหลักที่อ่านได้
การไม่พบหรือการจับคู่ MFT ที่กำกวมจะทำให้เกิดคำเตือนและจำนวน `Logs without a matching MFT`
ในส่วนสรุปด้วย แต่ไม่นับเป็นความล้มเหลวในการวิเคราะห์

## ฟิลด์เอาต์พุต { #output-fields }

### ตัวตน ความสมบูรณ์ และที่มาของระเบียน { #record-identity-integrity-and-provenance }

| ฟิลด์ | ความหมายและการตีความ |
|---|---|
| `SourceFile`, `FileOffset` | พาธอินพุตและ byte offset ที่อ่านส่วนหัวของระเบียนนี้ ใช้เพื่อตรวจสอบไบต์ |
| `LSN` | log sequence number ของระเบียน ค่าที่มีขนาดใหญ่เป็นจำนวนเต็ม ไม่ใช่ timestamp |
| `PreviousLSN`, `UndoNextLSN` | ลิงก์ไปยังระเบียนก่อนหน้าและระเบียนสำหรับย้อนกลับ ศูนย์หมายถึงไม่มีลิงก์ ลิงก์ที่ไม่เป็นศูนย์อาจชี้ออกไปนอกข้อมูลที่เก็บไว้ |
| `TransactionID`, `TransactionFirstLSN` | ช่องใน transaction table และ LSN แรกสุดที่เชื่อมโยงกับธุรกรรมนี้ ช่องเพียงอย่างเดียวไม่ได้มีค่าเฉพาะตัว |
| `RecordType` | `ClientRecord` (update), `ClientRestart` (checkpoint) หรือ `Other` |
| `ClientIndex`, `ClientSequence` | ตัวตนของ log client ซึ่งโดยปกติคือ NTFS ค่าเหล่านี้ไม่ใช่บัญชีผู้ใช้หรือ process ID |
| `Flags`, `SpansPages` | แฟล็กดิบของระเบียน และข้อมูลของ client ต่อเนื่องไปยังหน้าอื่นหรือไม่ การข้ามหน้าเป็นเรื่องปกติ |
| `Truncated` | เป็น true หากพาร์เซอร์พบว่าส่วนต่อเนื่องของ LSN นี้ขาดหายไป ไม่ได้สรุปความเสียหายทั้งหมดของทั้งไฟล์ |
| `ClusterSize`, `ClusterSizeSource` | จำนวนไบต์ต่อคลัสเตอร์ที่ใช้คำนวณ offset ของเป้าหมาย แหล่งที่มาคือ `Caller`, `Checkpoint` หรือ `Inferred` หากทราบค่าจริงให้ระบุด้วย `--cluster-size` |
| `LogVersion` | เวอร์ชันจาก restart page ล่าสุดที่อ่านได้ โดยทั่วไปคือ `1.1` หรือ `2.0` เป็น null หากไม่มี restart page ที่อ่านได้หลงเหลืออยู่ |
| `MFT-Source` | MFT ที่ใช้เสริมข้อมูลพาธปัจจุบัน (ถ้ามี) ต้องเป็นของวอลุ่มเดียวกัน |

ใน JSON/JSONL ของ `auto-parse` จะมีการเพิ่ม `Comp` และ `Artifact` เช่นเดียวกับพาร์เซอร์อื่น
log `.txt` ของงานเก็บข้อความวินิจฉัย ส่วน manifest ระบุว่างานสำเร็จ ล้มเหลว หรือถูกข้าม
ความเสียหายที่ยังกู้ได้อาจทำให้งานสำเร็จแต่มีจำนวน `Parse problems` ที่ไม่เป็นศูนย์ใน log ของงาน

### การดำเนินการและเป้าหมาย { #operations-and-target }

| ฟิลด์ | ความหมายและการตีความ |
|---|---|
| `RedoOperation`, `UndoOperation` | ชื่อการดำเนินการของ NTFS หรือเลขฐานสิบหกสำหรับรหัสที่ไม่รู้จัก redo อธิบายการอัปเดตที่ตั้งใจ ส่วน undo อธิบายการดำเนินการผกผัน |
| `TargetEntryNumber`, `TargetSequenceNumber` | สำหรับการดำเนินการกับ file record: หมายเลข MFT record ที่ได้รับผลกระทบ โดยไม่อนุมานหมายเลข sequence กรณีอื่น: file reference ที่เป็นเจ้าของ open attribute ที่ระบุได้ การอัปเดต directory index แบบ inline มีเป้าหมายคือไดเรกทอรี ไม่ใช่ child ที่ entry อธิบาย |
| `TargetAttributeType`, `TargetAttributeName` | ชนิดที่เป็นตัวเลขและชื่อของ open attribute ที่การดำเนินการใช้ สำหรับการดำเนินการกับ file record โดยปกติคือ `$DATA` ของ `$MFT` ชนิดที่พบบ่อย: 16 (`$STANDARD_INFORMATION`), 48 (`$FILE_NAME`), 128 (`$DATA`), 144 (`$INDEX_ROOT`), 160 (`$INDEX_ALLOCATION`), 176 (`$BITMAP`) |
| `TargetPath` | พาธ MFT ปัจจุบันเมื่อทราบ reference แบบเต็มของเป้าหมาย หรือส่วนประกอบที่ไม่ทราบ ว่างเปล่าสำหรับการดำเนินการกับ file record ที่ไม่ทราบรุ่น (generation) ในอดีต หมายเลข record เพียงอย่างเดียวต้องไม่ถูกแปลงเป็นผู้ครอบครองปัจจุบัน |
| `OpenAttributeEntryNumber`, `OpenAttributeSequenceNumber`, `OpenAttributePath` | ไฟล์ที่เป็นเจ้าของ open attribute ที่การดำเนินการใช้เขียน พร้อมพาธ MFT ปัจจุบัน สำหรับการดำเนินการกับ file record ฟิลด์เหล่านี้เก็บบริบทของ `$MFT` แยกจาก record ที่ได้รับผลกระทบ |
| `TargetAttribute` | offset ใน open-attribute table ของ log ซึ่งแตกต่างจากชนิดของ attribute หรือตัวระบุใน MFT |
| `TargetInferred` | เป็น true หากการแมปเป้าหมายมาจากช่วงอื่นของ log เป้าหมายที่เป็น null ยังคงไม่ถูกระบุแม้แฟล็กนี้จะเป็น false |
| `TargetVCN`, `Lcns` | virtual cluster ในเป้าหมายและหมายเลข physical cluster มีประโยชน์สำหรับการตรวจสอบระดับต่ำมากกว่าการระบุเวลา |
| `RecordOffset`, `AttributeOffset` | offset ของ attribute/entry และของการอัปเดตภายในนั้น |
| `ClusterBlockOffset`, `TargetBlockSize` | offset และขนาดเป้าหมายในหน่วย 512 ไบต์ |
| `OffsetInTarget` | byte offset ที่คำนวณได้ใน file record หรือ non-resident stream ที่ได้รับผลกระทบ ขึ้นอยู่กับขนาดคลัสเตอร์และขนาด MFT record |
| `RedoLength`, `UndoLength` | ความยาวข้อมูลที่ประกาศไว้ การดำเนินการแบบ zeroing อาจประกาศความยาวโดยไม่มีไบต์ข้อมูลจริงจำนวนเท่านั้น |
| `RedoBytesAvailable`, `UndoBytesAvailable` | จำนวนไบต์ข้อมูลที่มีจริงสำหรับการถอดรหัส ให้เปรียบเทียบกับความยาวที่ประกาศไว้และความหมายของการดำเนินการ |

### คอลัมน์เมทาดาทาไฟล์เพื่อความสะดวก { #convenient-file-metadata-columns }

คอลัมน์เหล่านี้แสดงชื่อไฟล์ที่ไม่ใช่ DOS ที่ต้องการหนึ่งชื่อจากฝั่ง redo และใช้ฝั่ง undo เป็นทางเลือกสำรอง
หากทั้งสองฝั่งไม่มีชื่อไฟล์ ฟิลด์การอัปเดต timestamp จะเลือก timestamp ฝั่ง redo ก่อน แล้วจึงเป็น undo
การอัปเดต directory entry ยังแสดงเวลา ขนาด และแฟล็กโดยตรง โดยเลือก redo ก่อน
undo แอตทริบิวต์ `$DATA` ที่ถอดรหัสแล้วจะแสดงขนาดแม้ไม่มีชื่อไฟล์ ขนาดของ `$DATA` ที่ไม่มีชื่อใน
file record ที่ถอดรหัสแล้วจะมีลำดับความสำคัญเหนือขนาดในชื่อไฟล์ซึ่งอาจล้าสมัย
คอลัมน์เหล่านี้มีไว้ช่วยในการเรียกดู ให้ใช้ฟิลด์แยกของแต่ละฝั่งสำหรับการเปรียบเทียบก่อน/หลังอย่างครบถ้วน

| ฟิลด์ | ความหมายและการตีความ |
|---|---|
| `MetadataSide` | `Redo` หรือ `Undo` ระบุฝั่งที่ให้ข้อมูลแก่คอลัมน์ชื่อไฟล์/timestamp เพื่อความสะดวก |
| `EntryNumber`, `SequenceNumber` | ไฟล์ที่อธิบายโดยเมทาดาทาชื่อไฟล์/file record/attribute ที่ถอดรหัสแล้ว หรือการอัปเดต timestamp ของ `$STANDARD_INFORMATION` บางส่วน พร้อมหมายเลข sequence เมื่อทราบ ว่างเปล่าสำหรับการอัปเดต directory entry ที่ไม่ระบุ child และการอัปเดตขนาด/run-list แบบดิบ ใช้ `TargetEntryNumber` สำหรับ record ที่ได้รับผลกระทบ |
| `Filename` | ชื่อที่ถอดรหัสแล้วที่ต้องการ ชื่อทั้งหมดยังคงอยู่ในอาร์เรย์ของแต่ละฝั่ง รวมถึง DOS alias และ hard link |
| `ParentEntryNumber`, `ParentSequenceNumber`, `ParentPath` | reference ของ parent และพาธ MFT ปัจจุบัน ไม่ใช่การสร้างพาธในอดีตขึ้นใหม่ |
| `CreatedOn`, `ContentModifiedOn`, `RecordModifiedOn`, `LastAccessedOn` | เวลาของเมทาดาทาเป็น UTC ละเอียดถึง 100 ns ไม่ใช่ timestamp ของธุรกรรม เวลาที่ไม่มีหรือไม่ถูกต้องจะยังคงว่างเปล่า/เป็น null |
| `FileSize`, `FileAttributes` | ขนาดและแฟล็กตัวเลขของ Windows จากเมทาดาทาที่เลือก ค่าจาก directory entry อาจล้าสมัย แอตทริบิวต์ `$DATA` และ file record ที่ถอดรหัสแล้วอาจให้ขนาดที่ตรงกว่า |
| `AttrType` | ชนิดที่เป็นตัวเลขของ attribute record ที่ถอดรหัสแล้ว เป็น 48 สำหรับการอัปเดตเมทาดาทา directory entry ที่ถอดรหัสแล้ว แยกจาก open attribute ที่การดำเนินการมีเป้าหมาย |

### เมทาดาทาและข้อมูล redo/undo ฉบับสมบูรณ์ { #complete-redoundo-metadata-and-payloads }

| ฟิลด์ | ความหมายและการตีความ |
|---|---|
| `RedoFileNames`, `UndoFileNames` | อาร์เรย์ของ `$FILE_NAME` ทุกรายการที่ถอดรหัสได้ในฝั่งนั้น โดยแยกชื่อที่ถูกเพิ่ม/ใหม่ออกจากชื่อที่ถูกลบ/เก่า |
| `RedoTimes`, `UndoTimes` | timestamp ของ `$STANDARD_INFORMATION` ที่แต่ละฝั่งเขียน การอัปเดตบางส่วนจะไม่แสดง timestamp ที่ไม่เปลี่ยนแปลงแทนที่จะคาดเดาค่า |
| `RedoDecoded`, `UndoDecoded` | ข้อมูล file record, attribute, index entry หรือ directory entry แบบมีโครงสร้าง ระบุด้วย `Kind` ค่า null อาจหมายถึงการดำเนินการที่ไม่รองรับหรือไบต์ไม่เพียงพอ |
| `JournalWriteOffset`, `JournalWriteLength` | offset และจำนวนไบต์ของข้อมูล redo ที่เขียน `$DATA:$J` ใช้ระบุแหล่งที่เป็นไปได้ของระเบียน USN ที่มิฉะนั้นจะสูญหาย ไม่ได้ส่งออกบันทึกที่สร้างขึ้นใหม่ |
| `Checkpoint` | เวอร์ชันของ checkpoint, LSN เริ่มต้น ขนาดคลัสเตอร์ และ LSN/ความยาวของ dump ของ open-attribute, attribute-name, dirty-page และ transaction table |
| `OtherRecordType` | ชนิดดิบของ client record สำหรับเนื้อหาที่ไม่รู้จักหรือถอดรหัสไม่ได้ |
| `RedoData`, `UndoData`, `OtherData` | ข้อมูล hex จะมีเฉพาะเมื่อใช้ `--raw-data` มิฉะนั้นจะเป็น null/ว่างเปล่า ให้เก็บไว้เพื่อตรวจสอบโครงสร้างที่ไม่รองรับหรือเสียหาย |

ออบเจ็กต์ชื่อไฟล์แต่ละรายการประกอบด้วย reference ของไฟล์และของ parent (`Raw`, `EntryNumber`,
`SequenceNumber`), `Name`, `NameEscaped`, `Namespace`, `ParentPath` ปัจจุบัน, timestamp สี่ค่า,
`Size`, `AllocatedSize`, `Attr` และ `ReparseValue` `NameEscaped` ระบุ UTF-16 ที่ไม่ถูกต้อง
ซึ่งแสดงด้วย escape ค่า reparse อาจเป็น reparse tag หรือขนาดของ extended attribute

ออบเจ็กต์ timestamp เก็บทั้ง `Utc` และจำนวนเต็ม `Filetime` ไว้ ค่า sentinel หรือค่าที่ไม่ถูกต้องอาจ
มีการแสดง UTC เป็น null ขณะที่ยังคงเก็บจำนวนเต็มต้นฉบับไว้ `RedoTimes` และ `UndoTimes`
มี `Created`, `Modified`, `Changed` และ `Accessed` สมาชิกที่ไม่มีอยู่จะมีค่าเป็น null

ออบเจ็กต์ `FileRecord` ที่ถอดรหัสแล้วประกอบด้วยหมายเลข sequence จำนวน hard link แฟล็ก base reference
มีเพียงส่วนหัวที่หลงเหลืออยู่หรือไม่ และ attribute ที่มีอยู่ attribute ที่ถอดรหัสแล้วแต่ละรายการเก็บรักษา
ชนิด ชื่อ สถานะ resident แฟล็ก ID ขนาด และขนาดที่จัดสรรไว้ `$STANDARD_INFORMATION` เพิ่ม
เวลาสี่ค่า แอตทริบิวต์ owner/security ID, quota และ USN ซึ่ง ID เหล่านี้ต้องแปลงค่าแยกต่างหากและ
ไม่ใช่ข้อพิสูจน์ว่าผู้ใช้รายใดเปลี่ยนแปลงไฟล์ index entry ประกอบด้วยแฟล็ก ความยาวคีย์ และ
ชื่อไฟล์ (ถ้ามี) การอัปเดต directory entry จะเก็บรักษาเวลาสี่ค่า ขนาด แอตทริบิวต์ และค่า reparse
ไว้แม้ชื่อเองจะไม่ได้ถูกบันทึกไว้

## ขั้นตอนการตรวจสอบ { #investigation-workflow }

1. ตรวจทานข้อความวินิจฉัยเกี่ยวกับ restart page ที่ขาดหาย หน้าที่ฉีกขาด และระเบียนที่ถูกตัดทอน
2. ค้นหาชื่อไฟล์หรือนามสกุลที่น่าสงสัยทั้งใน `RedoFileNames` และ `UndoFileNames`
3. จับคู่หมายเลข entry **และ sequence** จากนั้นตรวจสอบสาย LSN และการดำเนินการของธุรกรรม
4. เปรียบเทียบ timestamp ของ redo และ undo เพื่อหาการเปลี่ยนแปลง การคัดลอก การแตกไฟล์ และการกู้คืนตามปกติ
   ก็สามารถเปลี่ยน timestamp ได้เช่นกัน ความแตกต่างเป็นเบาะแส ไม่ใช่คำตัดสินว่าเกิด timestomping โดยอัตโนมัติ
5. เชื่อมโยงกับ[บันทึก USN](usn-journal.md), [$MFT](mft.md) และ [$I30](i30.md) ใช้ event
   log, [Prefetch](prefetch.md) และอาร์ติแฟกต์กิจกรรมของผู้ใช้เพื่อยืนยันการเรียกใช้งานหรือการระบุตัวผู้กระทำ

ตัวอย่างเช่น ค้นหาระเบียนทั้งหมดที่มีชื่อไฟล์ที่ถูกเข้ารหัสในฝั่งใดฝั่งหนึ่ง:

```bash
jq 'select(any(.RedoFileNames[]?, .UndoFileNames[]?;
  .Name | endswith(".encrypted"))) |
  {Lsn, TransactionId, TransactionFirstLsn, RedoOperation, UndoOperation,
   RedoFileNames, UndoFileNames}' logfile.jsonl
```

นี่คือการค้นหาชื่อไฟล์ ไม่ใช่เครื่องมือตรวจจับ ransomware ที่สมบูรณ์ หากต้องการตรวจดูข้อมูลของบันทึก
ให้ส่งออกด้วย `--raw-data` และเลือกระเบียนที่มี `JournalWriteOffset` ไม่เป็น null โดย hex ใน `RedoData`
แทนไบต์ที่เขียน ณ offset นั้น การวิเคราะห์และการตัดข้อมูลซ้ำของระเบียน USN ที่ฝังอยู่เหล่านั้น
เป็นอีกขั้นตอนหนึ่ง ปัจจุบัน Juntei ยังไม่ได้รวมระเบียนเหล่านี้เข้ากับเอาต์พุตของ `parse-ntfs usn-journal`

## รายละเอียดเพิ่มเติม { #further-details }

- [logfile crate ของ ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/logfile): โครงสร้างที่รองรับ ชุดข้อมูลสำหรับตรวจสอบความถูกต้อง และข้อจำกัดที่ทราบ
- [Sootmark ntfslog](https://github.com/Sootmark/ntfslog): การพัฒนาด้วย Rust ต้นทาง (upstream)
- [เอกสารรูปแบบ NTFS ของ libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation): หน้า log, fixup และโครงสร้างเมทาดาทา
- [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs): งานวิจัยและการพัฒนาการวิเคราะห์ NTFS ที่เป็นอิสระ
- [LogFileParser โดย Joakim Schicht](https://github.com/jschicht/LogFileParser): การวิเคราะห์ transaction log อย่างละเอียด
- [ntfs-linker โดย Stroz Friedberg](https://github.com/strozfriedberg/ntfs-linker): การเชื่อมโยงแหล่งข้อมูล NTFS เข้าเป็นประวัติของไฟล์
- [DFIR Artifact Museum](https://github.com/AndrewRathbun/DFIRArtifactMuseum): ตัวอย่าง NTFS จริงของ Windows
- [Velocidex go-ntfs](https://github.com/Velocidex/go-ntfs): การพัฒนา NTFS และวอลุ่มทดสอบสาธารณะ
