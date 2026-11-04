# บันทึก USN (`$UsnJrnl:$J`) { #usn-journal-usnjrnlj }

## USN Journal คืออะไร { #what-the-usn-journal-is }

NTFS บันทึกการเปลี่ยนแปลงของไฟล์และไดเรกทอรีไว้ใน Update Sequence Number (USN) Change Journal
เรคคอร์ดหนึ่งรายการสามารถอธิบายการสร้าง การขยาย/เขียนทับข้อมูล การเปลี่ยนแอตทริบิวต์หรือความปลอดภัย การเปลี่ยนชื่อ
การปิด และการลบ โดยจัดเก็บ file reference, parent reference, ชื่อ, เหตุผล (reason), แอตทริบิวต์,
timestamp และ USN ที่เพิ่มขึ้นอย่างต่อเนื่อง

journal นี้ยอดเยี่ยมสำหรับการสร้างลำดับการดำเนินการกับไฟล์ของผู้โจมตีที่เกิดขึ้นติดต่อกันขึ้นมาใหม่ โดยเฉพาะเมื่อ
ไฟล์ถูกลบไปแล้ว journal บันทึก **ว่าเมทาดาทา/เนื้อหามีการเปลี่ยนแปลง** ไม่ใช่ไบต์ที่เปลี่ยนแปลง

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> NTFS บันทึกเหตุผลการเปลี่ยนแปลงตามที่ระบุสำหรับ
ไฟล์ที่อ้างอิง ณ timestamp นั้น เหตุผลมักสะสมรวมกันจนถึงเรคคอร์ด `CLOSE`

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\$Extend\$UsnJrnl:$J
```

`$J` เป็น named stream ของเมทาดาทา NTFS ให้เก็บรวบรวมด้วยเครื่องมือนิติวิทยาศาสตร์ที่อ่านข้อมูลดิบได้ และให้เก็บรวบรวม
`$MFT` ของวอลุ่มเดียวกันด้วย เพราะเรคคอร์ด USN จัดเก็บ parent file reference แทนที่จะเป็นพาธที่สมบูรณ์

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- journal เป็นแบบวนรอบและมีขนาดจำกัด เรคคอร์ดเก่าจะถูกเขียนทับ
- `FILE_DELETE` บอกว่า NTFS ประมวลผลการลบแล้ว เนื้อหาอาจยังกู้คืนได้จากที่อื่น
- เหตุผลเช่น `DATA_OVERWRITE` ไม่ได้เปิดเผยไบต์ โพรเซส หรือผู้ใช้ที่รับผิดชอบ
- เหตุผลหลายรายการอาจรวมอยู่ในเรคคอร์ดเดียว `CLOSE` มักเป็นจุดสิ้นสุดของการเปลี่ยนแปลงที่สะสมไว้
- พาธเป็นสิ่งที่สร้างขึ้นใหม่ ไดเรกทอรีแม่อาจถูกเปลี่ยนชื่อ ถูกลบ หรือถูกนำกลับมาใช้ใหม่แล้ว
- sequence number ของ file reference มีความสำคัญ entry number เพียงอย่างเดียวอาจเชื่อมโยงไปยังรุ่น (generation) ที่ผิด
- การกู้คืนแบบ carving สามารถกู้เรคคอร์ดที่ถูกต้องได้โดยไม่มีบริบทของ journal ดั้งเดิม ให้รายงานความแตกต่างนี้ด้วย

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs usn-journal <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $J files
  -f, --file <FILE>      Parse one $J, or any file with --carve
  -m, --mft <FILE>       Use the matching volume's $MFT for paths
      --carve            Search a disk image/unallocated file for records

Output:
  -o, --output <PATH>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs usn-journal -f 'C/$Extend/$J' -m 'C/$MFT' -o usn.csv
juntei parse-ntfs usn-journal -d evidence/ -o usn-jsonl -t jsonl
juntei parse-ntfs usn-journal -f unallocated.bin --carve -o carved-usn.csv
```

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Name` | ชื่อไฟล์/ไดเรกทอรีที่จัดเก็บในเรคคอร์ด USN นี้ ไม่ใช่พาธที่สมบูรณ์ |
| `Extension` | นามสกุลที่แยกจาก `Name` เพื่อความสะดวก อย่าใช้เป็นชนิดเนื้อหาที่เชื่อถือได้ |
| `EntryNumber`, `SequenceNumber` | NTFS reference ของไฟล์ ใช้ทั้งสองค่าเพื่อระบุรุ่นของเรคคอร์ด |
| `ParentEntryNumber`, `ParentSequenceNumber` | NTFS reference ของไดเรกทอรีแม่ ณ เวลาที่เปลี่ยนแปลง |
| `ParentPath` | พาธของไดเรกทอรีแม่ที่ Juntei สร้างขึ้นใหม่โดยคำนึงถึงเวลา เครื่องหมาย unknown หมายความว่า resolve ลำดับบรรพบุรุษไม่ได้ |
| `UpdateSequenceNumber` | ค่าลำดับของ USN มีประโยชน์สำหรับการเรียงลำดับที่คงที่และการเชื่อมโยงกับ Amcache/MFT |
| `UpdateTimestamp` | timestamp ที่จัดเก็บในเรคคอร์ด V2/V3 ให้ตรวจสอบ timestamp จากบริบทการ carving ด้วยวิธีเดียวกับเรคคอร์ดอื่น |
| `UpdateReasons` | แฟล็กการเปลี่ยนแปลงที่มีชื่อหนึ่งรายการขึ้นไป เชื่อมด้วย `|` ให้ตีความการผสมกันของแฟล็กร่วมกับ `CLOSE` |
| `FileAttributes` | แอตทริบิวต์ NTFS ที่บันทึก ณ เวลาที่เปลี่ยนแปลง บิตที่ไม่รู้จักจะยังเป็นเลขฐานสิบหก |
| `OffsetToData` | byte offset ในอินพุต จำเป็นสำหรับการตรวจสอบความถูกต้องและเรคคอร์ดที่ได้จากการ carving |
| `SourceFile` | journal/อิมเมจอินพุต ใช้แยกแยะโฮสต์และวอลุ่ม |
| `SourceInfo` | แฟล็กแหล่งที่มาที่อธิบายว่าการเปลี่ยนแปลงเข้ามาใน journal อย่างไร โดยปกติเป็นบริบท ไม่ใช่การระบุตัวผู้กระทำ |
| `SecurityID` | ดัชนี security identifier ของ NTFS สำหรับเรคคอร์ด ไม่ใช่ SID ของผู้ใช้ |
| `Version` | เวอร์ชันของเรคคอร์ด USN Juntei เขียน V2/V3 ลง CSV ส่วน JSON เก็บ extent การติดตามช่วงของ V4 ไว้ |

JSON/JSONL แสดง reference แบบ 128 บิตของ V3 ได้อย่างถูกต้อง และเก็บรักษาเรคคอร์ดการติดตามช่วงของ V4 และ
extent ของเรคคอร์ดเหล่านั้น ให้ใช้รูปแบบนี้เมื่อฟิลด์ระดับต่ำที่เฉพาะเจาะจงตามเวอร์ชันมีความสำคัญ

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. กรองตามเวลา ชื่อ นามสกุล พาธของไดเรกทอรีแม่ หรือ file reference
2. เรียงลำดับตาม `UpdateSequenceNumber` ควบคู่กับเวลาเพื่อดูลำดับที่แน่นอนใน journal
3. จับคู่ `RENAME_OLD_NAME` กับ `RENAME_NEW_NAME` สำหรับการย้าย/การเปลี่ยนชื่อ
4. ใช้ `$MFT` เพื่อตรวจสอบสถานะของเรคคอร์ด พาธ ขนาด และ timestamp
5. เชื่อมโยงการระบุโพรเซสจาก event log, EDR, Prefetch, บริการ/งานตามกำหนดเวลา หรือ Defender

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Change Journals](https://learn.microsoft.com/en-us/windows/win32/fileio/change-journals)
- [Microsoft: USN_RECORD_V2](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2)
- [Microsoft: แฟล็กเหตุผลของ Change Journal](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ns-winioctl-usn_record_v2#members)
- [crate usn ของ ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser/tree/main/usn)
- [Sootmark usn](https://github.com/Sootmark/usn)
