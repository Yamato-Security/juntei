# ShellBags { #shellbags }

## ShellBags คืออะไร { #what-shellbags-are }

Explorer จดจำวิธีแสดงผลของแต่ละโฟลเดอร์ที่ผู้ใช้เปิด ได้แก่ มุมมอง ขนาดไอคอน ลำดับการเรียง และ
ตำแหน่งหน้าต่าง เพื่อให้หาการตั้งค่าของโฟลเดอร์ได้อีกครั้ง Explorer จะเก็บตัวตนของโฟลเดอร์ไว้ในรีจิสทรีของผู้ใช้
เป็นโครงสร้างต้นไม้ของ shell item ซึ่งเป็น item ชนิดเดียวกับที่อยู่ใน target ID list ของลิงก์ คือ ไดรฟ์ แล้วตามด้วย
แต่ละโฟลเดอร์ลงไปจนถึงโฟลเดอร์ที่ถูกเปิด โครงสร้างต้นไม้เหล่านี้คือ ShellBags

ShellBags ครอบคลุมโฟลเดอร์ที่เปิดใน Explorer และในกล่องโต้ตอบเปิดและบันทึก รวมถึงโฟลเดอร์บนไดรฟ์
แบบถอดได้ แชร์บนเครือข่าย อุปกรณ์พกพา (โทรศัพท์ กล้อง) ไฟล์บีบอัด ZIP และเซิร์ฟเวอร์ FTP
และหน้า Control Panel โดยยังคงอยู่หลังจากโฟลเดอร์ ไดรฟ์ หรือแชร์บนเครือข่ายหายไปแล้ว

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Explorer ของผู้ใช้นี้แสดงโฟลเดอร์นี้
(หรือโฟลเดอร์ที่อยู่ภายใต้โฟลเดอร์นี้) ShellBag ไม่ได้แสดงว่ามีการทำอะไรในโฟลเดอร์ และไม่ได้ระบุวันที่ของ
ทุกการเข้าชม: ดูเรื่องเวลาด้านล่าง

## ตำแหน่ง { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat
Local Settings\Software\Microsoft\Windows\Shell\BagMRU               (Windows Vista and later)
Wow6432Node\Local Settings\Software\Microsoft\Windows\Shell\BagMRU   (32-bit Explorer)

C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\Shell\BagMRU         (desktop and network items)
Software\Microsoft\Windows\ShellNoRoam\BagMRU   (Windows XP: local folders)
```

คีย์ `BagMRU` เก็บค่าที่มีหมายเลขกำกับ ซึ่งแต่ละค่าคือ shell item หนึ่งรายการ และมีค่า `MRUListEx` ที่ระบุ
ลำดับของค่าเหล่านั้น โดยรายการที่เปิดล่าสุดอยู่ก่อน แต่ละค่ามีคีย์ย่อยชื่อเดียวกันสำหรับ item
ที่อยู่ภายใต้ ดังนั้นพาธหนึ่งพาธคือ item ตั้งแต่ `BagMRU` ลงไปจนถึงค่าหนึ่ง `NodeSlot` ของคีย์ย่อยคือ
หมายเลขของคีย์ `Bags\<N>` ซึ่งเป็นที่เก็บการตั้งค่ามุมมองของโฟลเดอร์

ให้เก็บรวบรวม `UsrClass.dat` และ `NTUSER.DAT` พร้อม transaction log (`.LOG1`, `.LOG2`) ของทุก
โปรไฟล์ และไฮฟ์ `SOFTWARE` ของระบบ ซึ่งให้ SID ของผู้ใช้ `NTUSER.DAT`

## เวลาใดบอกอะไร { #which-times-say-what }

- **เวลาของ shell item** (`ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC`) คือ
  เวลาในระบบไฟล์ของโฟลเดอร์เอง (ความละเอียดแบบ FAT: 2 วินาที) ณ ตอนที่ Explorer เขียน item นั้น เวลาเหล่านี้
  ระบุวันที่ของโฟลเดอร์ ไม่ใช่การเข้าชม และจะไม่ถูกอัปเดตในการเข้าชมครั้งต่อ ๆ ไป
  นาฬิกา FAT ไม่มีการจัดเก็บค่า UTC offset หัวข้อ `UTC` ตามรูปแบบเดิมและส่วนต่อท้าย `Z` ที่แสดงเป็น
  ธรรมเนียมการแสดงผล ไม่ใช่หลักฐานของเขตเวลาของแหล่งข้อมูล `auto-parse --timezone` จะคงฟิลด์
  เหล่านี้ไว้ตามเดิม ให้ตีความโดยใช้บริบทของระบบต้นทาง
- **เวลาเขียนล่าสุดของคีย์** จะเปลี่ยนเมื่อ `MRUListEx` ของคีย์ถูกเขียนใหม่ คือเมื่อ item หนึ่งใน
  คีย์นั้นถูกเปิดและย้ายไปไว้ด้านหน้า ดังนั้นเวลาของคีย์ที่ค่าของ item อยู่
  (`ParentKeyLastWrittenUTC`) คือเวลาที่ item นั้นถูกเปิดครั้งล่าสุด **เฉพาะเมื่อเป็น item ล่าสุด
  ในคีย์นั้นเท่านั้น** (`IsMostRecentInParent` เป็น true, `MruPosition` 0) สำหรับ item อื่น ๆ จะเป็น
  เวลาที่ช้ากว่า คือเวลาที่ item พี่น้อง (sibling) ถูกเปิด
- คีย์ของ item เอง (`KeyLastWrittenUTC`) ระบุวันที่การเปิด child ล่าสุดของ item นั้นด้วยวิธีเดียวกัน

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **การไม่มีอยู่พิสูจน์ได้น้อย** Explorer เขียน ShellBags เมื่อมุมมองของโฟลเดอร์ถูกบันทึก โฟลเดอร์ที่
  ถูกดูเพียงชั่วครู่ หรือถูกเปิดด้วยวิธีอื่น (บรรทัดคำสั่ง สคริปต์ โปรแกรม) อาจไม่มี ShellBag เลย
  ผู้ใช้อาจลบคีย์เหล่านี้ได้
- **NTFS reference เป็นของโฟลเดอร์** (`MftEntry`, `MftSequence`) ณ ตอนที่ item ถูกเขียน
  ให้ค้นหาใน `$MFT` เพื่อดูว่าโฟลเดอร์เดิมยังคงอยู่ หรือไฟล์อื่นนำ entry นั้นกลับมาใช้ใหม่
- **ไดรฟ์แบบถอดได้** ถูกตั้งชื่อตามอักษรไดรฟ์ อักษรนั้นคืออักษรที่ไดรฟ์มีอยู่ในตอนนั้น
  ให้จับคู่กับประวัติ USB และ mounted device เพื่อระบุอุปกรณ์
- **ชื่อมาจาก item** โฟลเดอร์ที่ถูกเปลี่ยนชื่อภายหลังจะยังคงชื่อเดิมไว้ที่นี่ โฟลเดอร์ root และ known
  folder ถูกตั้งชื่อตามที่เครื่องมือนิติวิทยาศาสตร์ดิจิทัลทั่วไปใช้ (`This PC`, `Quick Access`, `Downloads`)
  `Guid` ให้ตัวระบุของโฟลเดอร์เหล่านั้น
- **ผลการค้นหา ไลบรารี และหน้า Control Panel** เป็นมุมมอง ไม่ใช่โฟลเดอร์บนดิสก์
  (`property_view`, `control_panel_item`, `control_panel_category`)
- **item ที่ไม่มีพาร์เซอร์ใดรู้จัก** จะเก็บไบต์ไว้ใน `DataRaw` โดยมี `ItemType` เป็น `unknown` และ
  class byte อยู่ใน `ItemClass` item ที่เสียหายหรือผิดปกติจะถูกรายงานใน `Problems` และไบต์ของ item เหล่านั้น
  ก็ถูกเก็บไว้เช่นกัน
- **ไฮฟ์ที่ dirty** อาจไม่มี item ล่าสุดหากไม่มี log Juntei จะเล่นซ้ำ (replay) log ที่อยู่ข้าง
  ไฮฟ์ เว้นแต่ระบุ `--no-logs` โดย `TransactionLogStatus` บอกว่าเรคคอร์ดมาจาก log หรือไม่

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shell-bags <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI names in shell items (default: 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shell-bags -d evidence/ -o shellbags.csv
juntei parse-registry shell-bags -f UsrClass.dat -t jsonl \
  | jq 'select(.ItemType == "volume" and .GUID == "{F5FB2C77-0E2F-4A16-A381-3E560C68BC83}")'
```

shell item ถูกถอดรหัสด้วย ys-lnk-parser ซึ่งเป็นพาร์เซอร์ shell item ของ `parse-file lnk`
สรุปผลจะแสดงรายการโฟลเดอร์ที่ถูกเปิดล่าสุด (เรียงจากใหม่ที่สุด) ตามเวลาเขียนล่าสุดของคีย์แม่
พร้อมผู้ใช้ของโฟลเดอร์เหล่านั้น

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath`, `HiveType` | โฮสต์ ชื่อโฟลเดอร์โปรไฟล์ SID ของผู้ใช้ (คีย์ root ของ `UsrClass.dat` ระบุไว้ สำหรับ `NTUSER.DAT` ใช้ `ProfileList` ของระบบ) ไฮฟ์และชนิดของไฮฟ์ |
| `BagKey` | คีย์ของ item ภายใต้ root ของไฮฟ์ (`…\BagMRU\4\3\0`) ค่าของ item อยู่ในคีย์ที่อยู่เหนือขึ้นไป |
| `MRU-Position`, `IsMostRecentInParent` | ตำแหน่งของ item ใน `MRUListEx` ของคีย์แม่ (0: ถูกเปิดล่าสุด) จะว่างเมื่อรายการไม่มี item นั้น |
| `NodeSlot` | หมายเลขของคีย์ `Bags\<N>` (การตั้งค่ามุมมองของโฟลเดอร์) |
| `Path` | item ตั้งแต่บนสุดของต้นไม้ลงมาจนถึง item นี้ เชื่อมด้วย `\` โดย `[unknown]` แทน item ที่ไม่มีค่า |
| `ItemType`, `ItemClass` | ชนิดของ item (`root_folder`, `volume`, `directory`, `file`, `network_location`, `uri`, `ftp_folder`, `compressed_folder`, `portable_device`, `portable_device_folder`, `users_files_folder`, `property_view`, `control_panel_item`, `control_panel_category`, `cd_burn`, `unknown`) และ class byte |
| `ItemName`, `GUID` | ชื่อ (ชื่อยาวของโฟลเดอร์) และ class identifier ของ root folder, Control Panel item หรือ delegate folder |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | **จาก shell item:** เวลาของโฟลเดอร์ ณ ตอนที่ item ถูกเขียน (FAT ความละเอียด 2 วินาที) |
| `MFT-Entry`, `MFT-Sequence` | **จาก shell item:** NTFS file reference ของโฟลเดอร์ |
| `ExtensionBlocks` | จำนวน extension block ที่ item มี |
| `KeyLastWrittenUTC` | **จากรีจิสทรี:** เวลาเขียนล่าสุดของคีย์ของ item เอง คือเวลาที่ child ล่าสุดถูกเปิด |
| `ParentKeyLastWrittenUTC` | **จากรีจิสทรี:** เวลาเขียนล่าสุดของคีย์ที่ค่าของ item อยู่ คือเวลาที่ item ถูกเปิดครั้งล่าสุด หาก `IsMostRecentInParent` เป็น true |
| `DataRaw` | ไบต์ของค่า (hex) สำหรับ item ที่ไม่เข้าใจอย่างสมบูรณ์หรือมีปัญหา |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่ `MRUListEx` ของคีย์ระบุแต่คีย์ไม่มี ค่าที่รายการไม่ได้ระบุ คีย์ย่อยที่ไม่มีค่าของตัวเอง ค่าที่ไม่ใช่ binary `NodeSlot` ที่มีชนิดไม่ถูกต้อง และ shell item list ที่เสียหาย |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. แสดงรายการไดรฟ์แบบถอดได้ (`volume` ที่มี `Guid` ของ Removable Drives) ตำแหน่งเครือข่าย โฟลเดอร์ FTP
   และอุปกรณ์พกพา รวมถึงโฟลเดอร์ที่อยู่ภายใต้สิ่งเหล่านั้น
2. มองหาโฟลเดอร์ที่น่าสนใจ: ตำแหน่งพักไฟล์ที่ผู้ใช้เขียนได้ (`Temp`, `ProgramData`, `Public`)
   โฟลเดอร์เครื่องมือและไฟล์บีบอัด โปรไฟล์ของผู้ใช้อื่น และโฟลเดอร์ที่ไม่มีอยู่แล้ว (ตรวจสอบ
   NTFS reference ใน `$MFT`)
3. ระบุวันที่การเข้าชมโดยใช้เฉพาะ `ParentKeyLastWrittenUTC` ของ item ที่ `IsMostRecentInParent` เป็น true
   ส่วนที่เหลือให้เชื่อมโยงกับไฟล์ LNK, Jump Lists และ RecentDocs ซึ่งระบุวันที่การเปิดไฟล์
4. ใช้ `parse-registry read --recover-deleted` สำหรับคีย์และค่า `BagMRU` ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [libyal libfwsi: รูปแบบ Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [libyal winreg-kb: Most recently used (BagMRU)](https://winreg-kb.readthedocs.io/en/latest/sources/explorer-keys/Most-recently-used.html)

การค้นหา SID จาก `ProfileList` ต้องมี SID ที่ไม่ซ้ำกันและถูกต้องตามไวยากรณ์เพียงหนึ่งค่า การจับคู่โฟลเดอร์โปรไฟล์ที่กำกวมและคีย์สำรอง เช่น `S-1-...bak` จะทำให้ `UserSid` เป็นค่าที่ไม่ทราบ
