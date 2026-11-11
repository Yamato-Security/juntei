# Jump Lists { #jump-lists }

## Jump Lists คืออะไร { #what-jump-lists-are }

Jump Lists เป็นข้อมูลเบื้องหลังเมนูที่แสดงจากรายการแอปพลิเคชันบนแถบงาน (taskbar) และเมนู Start โดย
Automatic Jump Lists เป็นประวัติที่ Windows จัดการเองของรายการที่แอปพลิเคชันใช้ ส่วน Custom Jump Lists เป็น
หมวดหมู่และงาน (task) ที่แอปพลิเคชันกำหนดเอง ทั้งสองแบบสามารถฝังโครงสร้าง LNK ที่มีพาธเป้าหมาย เวลา
บริบทวอลุ่ม/เครือข่าย MFT reference และอาร์กิวเมนต์สำหรับเรียกใช้ไว้ได้

เรคคอร์ดแบบอัตโนมัติสามารถเปิดเผยไฟล์หรือโฟลเดอร์ที่แอปพลิเคชันเปิด ลำดับ/เวลาการใช้งานล่าสุด
จำนวนการโต้ตอบ และสถานะการปักหมุด ส่วนลิสต์แบบกำหนดเองสามารถเปิดเผยหน้า "ที่ปิดล่าสุด" หรืองานของแอปพลิเคชัน
ซึ่งรวมถึงอาร์กิวเมนต์คำสั่งที่เป็นอันตรายได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> แอปพลิเคชันเจ้าของได้บันทึกหรือแสดง
รายการ/งานนั้นใน Jump List พฤติกรรมแตกต่างกันไปตามแอปพลิเคชัน ให้ยืนยันการเรียกใช้งาน/การเข้าถึงจริง

## ตำแหน่งและการเก็บรวบรวม { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\AutomaticDestinations\*.automaticDestinations-ms
%APPDATA%\Microsoft\Windows\Recent\CustomDestinations\*.customDestinations-ms
```

คำนำหน้าชื่อไฟล์แบบเลขฐานสิบหกคือ AppID ให้เก็บรวบรวมทั้งสองไดเรกทอรีและเก็บรักษาพาธ
โปรไฟล์ผู้ใช้ไว้ ลิสต์แบบอัตโนมัติเป็นคอนเทนเนอร์ OLE Compound File ที่มีสตรีม `DestList` และสตรีม LNK
ที่มีหมายเลขกำกับ ส่วนลิสต์แบบกำหนดเองใช้รูปแบบหมวดหมู่/ลิงก์ของตัวเอง

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- คำอธิบาย AppID มาจากการจับคู่ที่รู้จัก AppID ที่ไม่รู้จักไม่ได้น่าสงสัยในตัวเอง
- ฟิลด์ `LastModified` / เวลาใช้งานล่าสุดเป็นของรายการใน DestList ซึ่งต่างจากเวลาของเป้าหมายที่ฝังอยู่
- จำนวนการโต้ตอบมีเฉพาะในเวอร์ชันที่รองรับ และไม่ใช่จำนวนการเปิดทั้งหมด
- การปักหมุดทำให้รายการคงอยู่เกินกว่าลำดับความล่าสุดตามปกติ การล้างข้อมูลหรือการตั้งค่าแอปพลิเคชันอาจลบรายการอื่นออกไป
- ลิสต์อาจมีสตรีมที่ล้าสมัย/ไม่ได้ผูกอยู่ หรือไบต์จากลิสต์แบบกำหนดเองที่ยาวกว่าซึ่งมีอยู่ก่อนหน้า
- อาร์กิวเมนต์และพาธของ LNK ที่ฝังอยู่เป็นข้อมูลที่ผู้โจมตีควบคุมได้ การเชื่อมโยงกับแอปพลิเคชันเป็นเพียงบริบท ไม่ใช่ความน่าเชื่อถือ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file jump-lists <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find automatic/custom destination files
  -f, --file <FILE>      Parse one Jump List
      --code-page <CP>   ANSI code page for embedded links

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file jump-lists -d evidence/ -o jump-list-csv
juntei parse-file jump-lists -f sample.automaticDestinations-ms -t jsonl | jq .
```

เมื่อใช้ `auto-parse` ตาราง CSV คือ `FileAndFolderOpening/JumpLists-AutomaticDestinations.csv`
และ `FileAndFolderOpening/JumpLists-CustomDestinations.csv` ซึ่งอยู่ใต้คอมพิวเตอร์แต่ละเครื่องโดยตรง
log ที่ใช้ร่วมกันคือ `JumpLists-logs.txt` ในหมวดหมู่นั้น manifest จะแสดงพาธที่แน่นอน
ใน `OutputFiles` การส่งออก CSV แบบเดี่ยวจะคง `AutomaticDestinations.csv` และ
`CustomDestinations.csv` ไว้ในไดเรกทอรีที่คุณเลือก การส่งออกแบบอัตโนมัติในรูปแบบ JSON และ JSONL เป็น
ไฟล์ `JumpLists.json` หรือ `JumpLists.jsonl` ไฟล์เดียว โดยโครงสร้างเรคคอร์ดไม่เปลี่ยนแปลง

## ฟิลด์ CSV ของ AutomaticDestinations { #automaticdestinations-csv-fields }

| กลุ่มฟิลด์ | ฟิลด์และการตีความ |
|---|---|
| ต้นทาง | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: คอนเทนเนอร์ Jump List และเวลาในระบบไฟล์ของคอนเทนเนอร์ |
| แอปพลิเคชัน | `AppId`, `AppIdDescription`: AppID จากชื่อไฟล์และการจับคู่กับแอปพลิเคชันที่รู้จัก อย่าอนุมานตัวตนจากแฮชที่ไม่รู้จัก |
| คอนเทนเนอร์ | `HasSps`, `DestListVersion`, `LastUsedEntryNumber`: โครงสร้าง/เวอร์ชันของ DestList และบริบทของส่วนหัว |
| ลำดับ | `MRU`, `EntryNumber`: ตำแหน่งการใช้งานล่าสุดและตัวระบุรายการ/สตรีมแบบเลขฐานสิบหก |
| เมทาดาทาการใช้งาน | `CreationTime`, `LastModified`, `InteractionCount`, `PinStatus`, `PinPosition`: วงจรชีวิต/การใช้งาน/สถานะการปักหมุดใน DestList ให้ตีความตามเวอร์ชัน |
| โฮสต์/พาธ | `Hostname`, `MacAddress`, `Path`, `DisplayPath`: ตำแหน่งของเป้าหมายและบริบทการติดตาม โฮสต์/MAC ไม่ใช่ข้อมูลระบุตัวผู้กระทำเพียงลำพัง |
| ID การติดตาม | `FileBirthDroid`, `FileDroid`, `VolumeBirthDroid`, `VolumeDroid`: ใช้เชื่อมโยงเป้าหมายที่ถูกย้ายและการติดตามของ LNK ที่ฝังอยู่ |
| หมายเหตุ | `Notes`: แจ้งเกี่ยวกับสตรีมที่ไม่ได้ผูกอยู่ ข้อมูลหลงเหลือที่กู้คืนได้ หรือบริบทอื่นจากพาร์เซอร์ |
| LNK ที่ฝังอยู่ | `TargetCreated`, `TargetModified`, `TargetAccessed`, `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`, `DriveType`, `VolumeSerialNumber`, `VolumeLabel`, `LocalPath`, `CommonPath`, `TargetIDAbsolutePath`, `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`, `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `ExtraBlocksPresent`, `Args`: ตีความตามที่อธิบายไว้ใน [ทางลัด LNK](lnk.md) |
| LNK เพิ่มเติม | `NetworkPath`, `Name`, `IconLocation`: ฟิลด์เครือข่ายและการแสดงผลจากทางลัดที่ฝังอยู่ |
| การตรวจสอบความถูกต้อง | `Problems`: สตรีมที่หายไป การถูกตัดทอน จำนวนที่ไม่ตรงกัน หรือลิงก์ที่เสียหาย |

## ฟิลด์ CSV ของ CustomDestinations { #customdestinations-csv-fields }

เอาต์พุตแบบกำหนดเองเริ่มต้นด้วย `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`, `AppId`,
`AppIdDescription` และ `EntryName` (ชื่อหมวดหมู่หรือ `Tasks`) ตามด้วยฟิลด์ LNK ที่ฝังอยู่ทั้งหมด
ที่แสดงไว้ด้านบน แล้วตามด้วย `Notes`, `NetworkPath`, `Name`, `IconLocation` และ `Problems`

JSON/JSONL จะเก็บรักษาข้อมูลเพิ่มเติม ได้แก่ ส่วนหัว DestList, score/checksum/property store ของแต่ละรายการ,
`DestListPropertyStore`, สตรีมของ compound file, หมวดหมู่แบบกำหนดเอง, ไบต์หลังหมวดหมู่สุดท้าย
และฟิลด์ที่ซ้อนอยู่ของ LNK ที่ฝังอยู่ทุกรายการ

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ระบุผู้ใช้และโปรไฟล์ต้นทาง จากนั้นจับคู่ AppID กับแอปพลิเคชันอย่างระมัดระวัง
2. เรียงรายการแบบอัตโนมัติตาม `LastModified` และ `MRU` และแยกรายการที่ปักหมุดออก
3. ตรวจทานพาธ แชร์บนเครือข่าย serial ของวอลุ่มแบบถอดได้ MFT reference และอาร์กิวเมนต์
4. ตรวจสอบ `Notes` และลิงก์ที่ไม่ได้ผูกอยู่/หลงเหลือ เพราะรายการที่ถูกลบหรือรายการก่อนหน้าอาจยังคงอยู่ที่นั่น
5. เชื่อมโยงกับ RecentDocs, LNK ใน Recent แบบเดี่ยว, Prefetch, SRUM, `$MFT` และ event log

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: รูปแบบ Compound Binary File (MS-CFB)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-cfb/)
- [Microsoft: รูปแบบ Shell Link (MS-SHLLINK)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser)
- [โปรเจกต์ทดสอบ/อ้างอิงพาร์เซอร์ JumpList](https://github.com/AbdulRhmanAlfaifi/jumplist_parser)
