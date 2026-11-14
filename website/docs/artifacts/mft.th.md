# ตารางไฟล์หลัก (`$MFT`) { #master-file-table-mft }

## MFT คืออะไร { #what-the-mft-is }

NTFS แทนไฟล์และไดเรกทอรีทุกรายการด้วยเรคคอร์ดใน Master File Table เรคคอร์ดประกอบด้วย
แอตทริบิวต์ต่าง ๆ เช่น ชื่อ parent reference, timestamp, ขนาด, แฟล็ก, security identifier,
ข้อมูล reparse และเมทาดาทาของ data stream เนื้อหาของไฟล์ขนาดเล็กและ alternate data stream อาจเป็นแบบ
**resident** ซึ่งหมายความว่าไบต์ของข้อมูลถูกจัดเก็บไว้ภายในเรคคอร์ด MFT เอง

MFT เป็นรากฐานสำคัญเพราะครอบคลุมทั้งเรคคอร์ดที่ถูกจัดสรรและที่ถูกลบทั่วทั้งวอลุ่ม
สามารถแสดงได้ว่าไฟล์เคยมีอยู่ ถูกลิงก์ไว้ที่ใด ถูกลบหรือไม่ และเมทาดาทาของไฟล์
เปลี่ยนแปลงอย่างไร แม้ว่าการเรียกดูไดเรกทอรีตามปกติจะไม่แสดงไฟล์นั้นแล้วก็ตาม

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> เรคคอร์ดไฟล์ NTFS ที่มีเมทาดาทาตามที่แสดง
มีอยู่ ณ เวลาที่เก็บรวบรวม หรือหลงเหลืออยู่ในเรคคอร์ดที่ไม่ได้ถูกจัดสรร MFT เพียงอย่างเดียวไม่ได้
พิสูจน์การเรียกใช้งานหรือเจตนาของผู้ใช้

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\$MFT
```

`$MFT` เป็นเมทาดาทา NTFS ที่ได้รับการป้องกัน ให้เก็บรวบรวมด้วยเครื่องมือ forensic acquisition หรือเครื่องมือ triage ที่สามารถ
เข้าถึง NTFS แบบ raw ได้ การคัดลอกแบบ user-mode ทั่วไปโดยปกติไม่สามารถอ่านไฟล์นี้ได้ หากเป็นไปได้ให้เก็บคู่กับ `$J`, `$LogFile`
และเมทาดาทาของวอลุ่ม

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- เรคคอร์ดที่ถูกลบอาจถูกเขียนทับบางส่วน หรือหมายเลขเรคคอร์ดอาจถูกนำกลับมาใช้ใหม่
- `SN` ใช้แยกแยะรุ่น (generation) ของหมายเลข entry เดียวกัน ให้จับคู่ทั้ง entry **และ** sequence
- timestamp ของ `$STANDARD_INFORMATION` (`0x10`) และ `$FILE_NAME` (`0x30`) มีพฤติกรรมการอัปเดต
  ที่ต่างกัน และไม่มีชุดใดที่ปลอดภัยจากการปลอมแปลง
- `SI<FN`, `uSecZeros` และ `Copied` เป็นเบาะแส ไม่ใช่คำตัดสินอัตโนมัติว่ามีการทำ timestomping
- hard link ทำให้ไฟล์เรคคอร์ดเดียวมีแอตทริบิวต์ `$FILE_NAME` และพาธหลายรายการ
- NTFS สามารถเก็บชื่อได้หลายชื่อ (รวมถึง DOS 8.3) alternate data stream และ extension record
- การอัปเดตเวลาเข้าถึงอาจถูกปิดใช้งาน หน่วงเวลา หรือรวมเข้าด้วยกัน
- พาธเต็มถูกสร้างขึ้นใหม่จากเรคคอร์ดแม่ `[Orphaned]`, `[Unknown]` หรือ `PathUnknown`
  เป็นสัญญาณว่าลำดับบรรพบุรุษหายไป/ถูกนำกลับมาใช้ใหม่/เสียหาย ไม่ใช่ชื่อไดเรกทอรีจริง
- **slack** ของ FILE record (ไบต์ที่อยู่เลยขนาดที่ใช้งาน) อาจเก็บชื่อที่เรคคอร์ดเคยมีไว้ก่อนหน้า ได้แก่
  index entry ของไดเรกทอรีขนาดเล็กที่ดัชนีย้ายไปอยู่ใน index allocation (ส่วนใหญ่ยังคง
  ใช้งานอยู่ที่นั่น) ชื่อก่อนหน้า หรือผู้ครอบครองเดิมของเรคคอร์ดที่ถูกนำกลับมาใช้ใหม่ ชื่อที่กู้คืนได้
  ไม่มี `$STANDARD_INFORMATION` และส่วนหัว entry ของชื่ออาจถูกเขียนทับแล้ว เมื่อ
  ขนาดที่ใช้งานหรือแอตทริบิวต์ของเรคคอร์ดไม่ได้บอกว่า slack เริ่มต้นที่ใด จะไม่มีการกู้คืนสิ่งใดจากเรคคอร์ดนั้น
  และ `ParseErrors` ของแถวนั้นจะระบุ `slack not read` พร้อมเหตุผล

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs mft <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $MFT files
  -f, --file <FILE>      Parse one $MFT

Filtering:
  -r, --ranges <RANGES>  Parse selected entry numbers (example: 0-15,30)

Output:
  -o, --output <PATH>                   Save records
  -t, --output-type <TYPE>              csv, ys-ntfs-parser-csv, json, jsonl
  -e, --extract-resident-streams <DIR>  Extract resident $DATA streams
  -R, --reconstruct-filesystem <DIR>  Recreate a tree; recovered content gets .dontrun
      --reconstruction-i30 <FILE>      Supplement the tree with an I30 file (repeatable)

CSV Output:
  -a, --all-times      Include every $FILE_NAME time
  -s, --short-names    Include DOS 8.3-name rows
      --cutoff <DATE>  Keep recent SI-created or SI-modified records

Records:
  -x, --recover-slack  Recover file names from FILE record slack

Resident Data:
  -i, --include-resident-data      Put resident $DATA in the CSV
  -E, --resident-extensions <EXT>  Only these extensions (ex: ps1,bat,vbs,js)
  -M, --resident-max-size <BYTES>  Only data up to this size

Inspection:
  -D, --dump-entry <ENTRY>           Show one entry in full detail
  -L, --list-directory <ENTRY|PATH>  List a directory's contents
```

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv
juntei parse-ntfs mft -f '$MFT' --cutoff '2026-10-20T00:00:00Z' -a -o recent.csv
juntei parse-ntfs mft -f '$MFT' -t jsonl -r 32000-32100 | jq .
juntei parse-ntfs mft -f '$MFT' -x -o mft-with-slack.csv
juntei parse-ntfs mft -f '$MFT' -i -E ps1,bat,vbs,js -o scripts.csv
juntei parse-ntfs mft -f '$MFT' -D 0x270-5
juntei parse-ntfs mft -f '$MFT' -L '\Users\alice\Downloads'
```

ห้ามเรียกใช้ resident stream ที่แยกออกมาด้วย `-e` เด็ดขาด Juntei ตั้งชื่อไฟล์เหล่านั้นเป็น `.dontrun` ด้วยเหตุผลนี้
คอลัมน์ resident data ของ `-i` คือไบต์ชุดเดียวกัน ให้ถือว่าเป็นเนื้อหาที่ไม่น่าเชื่อถือ

`-D` พิมพ์ entry หนึ่งรายการ: ส่วนหัว แอตทริบิวต์ทุกรายการ (พร้อม data run) และชื่อใน slack ตามที่
`-t json -x` เขียน แล้วตามด้วยไบต์ของเรคคอร์ด `-L` แสดงรายการทุกชื่อในไดเรกทอรี ทั้งที่ใช้งานอยู่และ
ถูกลบ พร้อมหมายเลข entry และ sequence ชนิด ขนาด และเวลาที่แก้ไข ไดเรกทอรีระบุได้ด้วย
entry (`30`, `0x1E`, `30-1`) หรือพาธ (`\Windows\System32`, `C:/Windows`; ไม่สนใจตัวพิมพ์เล็ก/ใหญ่)

ตัวอย่างนี้สร้างพาธไฟล์ที่เปิดดูได้ เนื้อหา non-resident เป็นไฟล์ว่าง รายละเอียด: [การสร้างระบบไฟล์ขึ้นใหม่](../commands/auto-parse.md#filesystem-reconstruction).

```bash
juntei parse-ntfs mft -f '$MFT' -R reconstructed
```

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `EntryNum` (Entry Number) | หมายเลขเรคคอร์ด MFT ให้ใช้ร่วมกับ `SN` เพราะหมายเลขเรคคอร์ดถูกนำกลับมาใช้ใหม่ |
| `SN` (Sequence Number) | ตัวนับรุ่นที่ NTFS reference ใช้ ค่าที่ไม่ตรงกันอาจเผยความสัมพันธ์ที่ล้าสมัย/ถูกลบ |
| `InUse` | แฟล็กการจัดสรร ณ เวลาที่เก็บรวบรวม `false` หมายถึงเรคคอร์ดว่าง/ถูกลบ ไม่ได้หมายความว่าไบต์ทั้งหมดยังสมบูรณ์ |
| `ParentEntryNum` (ParentEntryNumber), `ParentSN` (Parent Sequence Number) | reference ของไดเรกทอรีแม่จาก `$FILE_NAME` ให้ตรวจสอบทั้งสองส่วน |
| `ParentPath` | พาธแม่ที่สร้างขึ้นใหม่ เครื่องหมาย unknown/orphan แสดงลำดับบรรพบุรุษที่ระบุไม่ได้ |
| `File` (Filename) | ชื่อจาก `$FILE_NAME` หรือ stream ของแถวปัจจุบัน เรคคอร์ดเดียวอาจมีหลายแถว |
| `Ext` (Extension) | นามสกุลที่แยกจากชื่อเพื่อความสะดวก ไม่ใช่การระบุชนิดไฟล์ที่เชื่อถือได้ |
| `Size` (File Size) | ขนาดเชิงตรรกะของแถว/stream การจัดสรรแบบ sparse/compressed อาจแตกต่างออกไป |
| `RefCnt` (Reference Count) | จำนวน hard-link reference ของ NTFS ให้เปรียบเทียบกับจำนวนชื่อที่หลงเหลืออยู่อย่างระมัดระวัง |
| `ReparseTarget` | เป้าหมายที่ถอดรหัสแล้วของ reparse point ที่รองรับ เช่น junction/symlink พาธอาจถูกเปลี่ยนทิศทาง |
| `Dir` (Directory) | เรคคอร์ดมีแฟล็กไดเรกทอรี |
| `HasADS` (Has Alternate Data Streams) | เรคคอร์ดมี alternate data stream ที่มีชื่อ ซึ่งไม่ได้ทำให้เป็นอันตราย |
| `IsADS` (Is Alternate Data Stream) | แถวนี้แทน alternate data stream ที่มีชื่อ แทนที่จะเป็นข้อมูลไฟล์ที่ไม่มีชื่อ |
| `SI<FN` (Standard Information Before File Name) | เวลาสร้างของ SI มาก่อนเวลาสร้างของ FN ที่ต่างกัน ให้ตรวจสอบการคัดลอก/timestomping และสาเหตุที่ถูกต้องตามกฎหมาย |
| `uSecZeros` (Microsecond Zeros) | เวลาสร้าง/แก้ไขของ SI ตรงกับขอบเขตมิลลิวินาทีพอดี เป็นเพียงฮิวริสติกที่อ่อนสำหรับ timestomping |
| `Copied` | เวลาแก้ไขมาก่อนเวลาสร้าง มักสอดคล้องกับการคัดลอกไฟล์ ไม่ใช่หลักฐานของการกระทำของผู้โจมตี |
| `SI-Flags` (Standard Information Flags) | แอตทริบิวต์/แฟล็กของไฟล์ใน `$STANDARD_INFORMATION` แฟล็ก hidden/system อาจเป็นจุดเชื่อมโยง (pivot) ที่มีประโยชน์ |
| `Type` (Name Type) | namespace ของ `$FILE_NAME` (Win32, DOS, POSIX หรือแบบรวม) ใช้เพื่อหลีกเลี่ยงการนับชื่อซ้ำ |
| `DataModified (SI)`, `Accessed (SI)`, `MetadataChanged (SI)`, `Created (SI)` | เวลาของ SI เป็นฟิลด์ไทม์ไลน์ทั่วไป แต่เครื่องมือ user-mode สามารถเปลี่ยนแปลงได้หลายค่า |
| `DataModified (FN)`, `Accessed (FN)`, `MetadataChanged (FN)`, `Created (FN)` | เวลาของ FN อัปเดตต่างกันและโดยทั่วไป API ทั่วไปเปลี่ยนแปลงได้ยากกว่า แต่ก็ยังไม่ใช่ค่าที่เปลี่ยนแปลงไม่ได้ |
| `USN` (Update Sequence Number) | USN ล่าสุดที่เกี่ยวข้องกับแอตทริบิวต์ SI ของเรคคอร์ด ใช้เชื่อมโยงไปยัง `$J` หาก USN นั้นยังหลงเหลืออยู่ |
| `LogfileSN` (Logfile Sequence Number) | sequence number ของ NTFS `$LogFile` จากส่วนหัวของ FILE record มีประโยชน์ในการวิเคราะห์ธุรกรรมระดับต่ำ |
| `SID` (Security ID) | ดัชนีไปยัง NTFS `$Secure` ไม่ใช่ Windows SID ในตัวเอง ให้เชื่อมกับ `SecurityID` ของ [`parse-ntfs secure-sds`](secure-sds.md) เพื่อดูเจ้าของและ ACL ของไฟล์ |
| `ObjectID-FileDroid` | Object ID / ตัวระบุ distributed-link-tracking (ถ้ามี) ให้เชื่อมโยงกับ droid ของ LNK/Jump List |
| `LoggedUtilStream` | บริบทของ `$LOGGED_UTILITY_STREAM` ที่ถอดรหัสแล้ว (ถ้ารองรับ) ให้ตรวจทานค่าที่ไม่คุ้นเคย |
| `ZoneID-Contents` | เนื้อหา `Zone.Identifier` แบบ resident เช่น ZoneId และ URL ที่ดาวน์โหลด อาจไม่มีอยู่ ถูกลบ หรือเป็นแบบ non-resident |
| `SourceFile` | `$MFT` อินพุต จำเป็นอย่างยิ่งเมื่อรวมข้อมูลหลายโฮสต์หรือหลายวอลุ่ม |
| `ResidentDataBase64`, `ResidentDataHex`, `ResidentDataASCII` | เมื่อใช้ `-i`: เนื้อหาของ resident `$DATA` ของแถว (ของไฟล์ หรือของ alternate data stream) ในรูป Base64 ไบต์ (`41-42`) และข้อความเมื่อเป็น ASCII ที่พิมพ์ได้ จะว่างเมื่อข้อมูลเป็นแบบ non-resident หรือถูกกรองออกโดย `-E`/`-M` |
| `ParseErrors` | ความเสียหายหรือความสัมพันธ์ที่ไม่รองรับซึ่งส่งผลต่อแถวนี้ ห้ามทิ้งคอลัมน์นี้ระหว่างการกรอง |
| `FromSlack` | เมื่อใช้ `-x`: `True` สำหรับชื่อที่กู้คืนจาก slack ของ FILE record โดย `EntryNum` และ `SN` คือไฟล์ที่ชื่อนั้นชี้ไป (จะว่างเมื่อส่วนหัว entry ถูกเขียนทับ) timestamp ที่กู้คืนได้อยู่ในคอลัมน์ `(FN)` และ `InUse` เป็น `False` |
| `SlackRecord`, `SlackOffset` | เมื่อใช้ `-x`: FILE record ที่ slack เก็บชื่อนั้นไว้ โดยระบุตามตำแหน่งใน `$MFT` แทนที่จะเป็นหมายเลขเรคคอร์ดที่เขียนอยู่ภายใน (เป็นของ extension record เอง แม้ว่าชื่อจะแสดงพร้อมกับ base record) และ byte offset ของชื่อใน `$MFT` `ParseErrors` ของแถว slack คือปัญหาของเรคคอร์ดนั้น เช่น `fixup values do not match` ซึ่งไบต์ที่กู้คืนได้อาจได้รับผลกระทบ |
| `CopyOfEntryInUse` | เมื่อใช้ `-x`: ไฟล์ที่ใช้งานอยู่ยังมีชื่อนี้ในไดเรกทอรีเดียวกัน (และเป็นไฟล์เดียวกัน เมื่อ reference ยังหลงเหลืออยู่) ชื่อที่ไม่มีไฟล์ที่ใช้งานอยู่ใดมีคือชื่อที่น่าสนใจ: ถูกลบ ถูกเปลี่ยนชื่อ หรือเป็นผู้ครอบครองเดิมของเรคคอร์ดที่ถูกนำกลับมาใช้ใหม่ |

คอลัมน์ timestamp เรียงตามลำดับ MACB (Modified, Accessed, Changed, Birth/creation):
`DataModified` SI/FN, `Accessed` SI/FN, `MetadataChanged` SI/FN แล้วตามด้วย `Created` SI/FN

`(SI)` หมายถึง `$STANDARD_INFORMATION` และ `(FN)` หมายถึง `$FILE_NAME` ชื่อเหล่านี้อธิบาย
แหล่งที่มาตามปกติของเรคคอร์ดที่สมบูรณ์ สำหรับแถวชื่อไฟล์ปกติที่ไม่มีแอตทริบิวต์ SI
fallback ที่มีอยู่จะใส่เวลาสร้างของ FN ไว้ใน `Created (FN)` และใส่ค่าเวลาแก้ไข/เปลี่ยนแปลงเรคคอร์ด/เข้าถึงของ FN
ไว้ใน `DataModified (SI)`, `MetadataChanged (SI)` และ `Accessed (SI)` ตามลำดับ
แถว slack ที่กู้คืนได้จะใส่เวลาทั้งสี่ไว้ในคอลัมน์ `(FN)` และปล่อยคอลัมน์ SI ว่างไว้
ค่า fallback ของแถวปกติ**ไม่ใช่**หลักฐาน SI ที่เป็นอิสระ ให้ใช้แอตทริบิวต์ JSON ดั้งเดิม
และที่มาของ slack เมื่อระบุแหล่งที่มาของ timestamp

`-t ys-ntfs-parser-csv` (เดิมคือ `ys-mft-parser-csv` ซึ่งยังคงยอมรับอยู่) ใช้ schema แบบเน้น entry
ของพาร์เซอร์ที่ได้รับการดูแล ชื่อคอลัมน์ที่อ่านง่ายเหล่านี้ใช้กับเอาต์พุต `-t csv` ค่าเริ่มต้น
รวมถึง `MFT.csv` ของ `auto-parse` ส่วน JSON/JSONL จะเก็บรักษาแอตทริบิวต์
ทุกรายการ (รวมถึง resident `$DATA` ในรูปเลขฐานสิบหก) และ `FullPath` ให้ใช้รูปแบบนี้เมื่อ CSV
แบบแบนสำหรับการตรวจสอบละเว้นโครงสร้างระดับต่ำ เมื่อใช้ `-x` แต่ละ entry ใน JSON จะมี
`SlackFileNames` ด้วย

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. กรองช่วงเวลา แต่ให้เก็บเรคคอร์ดที่ถูกลบและ timestamp ทั้งสองชุดไว้
2. ค้นหาพาธเต็ม นามสกุลซ้อนที่น่าสงสัย ADS ไดเรกทอรีสำหรับพักไฟล์ และแฮช/ชื่อที่รู้จัก
   เมื่อใช้ `-x` ให้รวมชื่อใน slack ที่ไม่มีไฟล์ที่ใช้งานอยู่ใดมี (`CopyOfEntryInUse` `False`) และเมื่อใช้
   `-i -E ps1,bat,vbs,js` ให้อ่านสคริปต์ขนาดเล็กได้โดยตรงจาก CSV
3. ใช้ entry/sequence และ `USN` เชื่อมโยงไปยัง USN Journal
4. เปรียบเทียบ MFT reference ของ Prefetch/LNK/Jump List กับเรคคอร์ดเป้าหมาย
5. เชื่อมโยงการลบกับการกักกันของ Defender หรือกิจกรรม `FILE_DELETE`/การเปลี่ยนชื่อใน USN

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Master File Table](https://learn.microsoft.com/en-us/windows/win32/fileio/master-file-table)
- [Microsoft Open Specifications: NTFS](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-ntfs/)
- [mft crate ของ ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/mft)
- [mft parser โดย Omer Ben-Amram](https://github.com/omerbenamram/mft)
- [เอกสารรูปแบบของ libfsntfs](https://github.com/libyal/libfsntfs/tree/main/documentation)
