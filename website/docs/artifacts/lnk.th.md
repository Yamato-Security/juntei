# ทางลัด LNK { #lnk-shortcuts }

## ไฟล์ LNK คืออะไร { #what-lnk-files-are }

ไฟล์ Windows Shell Link (`.lnk`) อธิบายเป้าหมาย (target) พร้อมอาร์กิวเมนต์สำหรับเรียกใช้ ไดเรกทอรีทำงาน ไอคอน
สถานะหน้าต่าง เมทาดาทาของเป้าหมาย ข้อมูลวอลุ่ม/เครือข่าย รายการใน shell namespace และตัวระบุ
distributed-link-tracking (ถ้ามี) Windows และแอปพลิเคชันสร้างไฟล์เหล่านี้ไว้ใน Recent Items,
Desktop, Start Menu และตำแหน่งอื่น ๆ ส่วนผู้โจมตีก็ส่งทางลัดที่เป็นอันตรายมาโดยตรงได้เช่นกัน

LNK ในโฟลเดอร์ Recent สามารถสนับสนุนข้อสรุปว่าผู้ใช้มีปฏิสัมพันธ์กับเป้าหมายที่ไม่มีอยู่แล้วในปัจจุบัน รวมถึง
ไดรฟ์แบบถอดได้หรือแชร์บนเครือข่าย ขณะที่ LNK ทั่วไปอาจมีอยู่โดยไม่เคยถูกคลิกเลย

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ทางลัดถูกสร้าง/อัปเดตพร้อมบริบทของเป้าหมาย
ที่บันทึกไว้ ตำแหน่งของทางลัดและอาร์ติแฟกต์แวดล้อมเป็นตัวกำหนดว่าทางลัดนั้นสนับสนุนข้อสรุปเรื่องการใช้งานหรือไม่

## ตำแหน่งและการเก็บรวบรวม { #locations-and-collection }

```text
%APPDATA%\Microsoft\Windows\Recent\*.lnk
%USERPROFILE%\Desktop\*.lnk
%APPDATA%\Microsoft\Windows\Start Menu\Programs\...
%PROGRAMDATA%\Microsoft\Windows\Start Menu\Programs\...
```

ให้ค้นหาทั่วทั้งโปรไฟล์ผู้ใช้ รวมถึงไฟล์บีบอัด/เนื้อหาอีเมลที่เก็บรวบรวมมา ไม่ใช่เฉพาะ `Recent` เท่านั้น
และให้เก็บรักษาเวลาและพาธของไฟล์ต้นทางเอง เพราะตำแหน่งที่พบมีผลต่อการตีความ

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- เวลา ขนาด MFT reference และข้อมูลวอลุ่มของเป้าหมายเป็นสแนปช็อตที่จัดเก็บไว้ใน LNK ไม่ใช่
  เมทาดาทาของไฟล์ทางลัดเอง และอาจล้าสมัยแล้ว
- `MachineID`, MAC และเวลาที่สร้าง tracker อธิบายบริบทของ distributed-link-tracking ซึ่งโดยทั่วไปคือ
  เครื่องที่สร้าง/อัปเดตลิงก์ ไม่จำเป็นต้องเป็นเครื่องที่คุณพบไฟล์นั้น
- node ของ UUID ที่ดูเหมือน MAC อาจมาจาก virtualization, การสุ่มค่า หรือตัวระบุที่ไม่ใช่ของเครือข่าย
- `Args` สามารถทำให้ไฟล์ปฏิบัติการที่ไม่เป็นอันตราย (`powershell.exe`, `rundll32.exe`, `mshta.exe`) กลายเป็นอันตรายได้
- สตริง ANSI ต้องใช้ code page ของระบบต้นทาง สำหรับลิงก์ภาษาญี่ปุ่นจำนวนมากให้ใช้ `--code-page 932`
- ทางลัดอาจถูกสร้างขึ้นโดยเจตนาหรือถูกคัดลอกมา อย่าถือว่าฟิลด์ของทางลัดเท่ากับการเรียกใช้งาน หากไม่มี Prefetch,
  Jump List, RecentDocs, event log หรือหลักฐานอื่นสนับสนุน

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file lnk <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.lnk
  -f, --file <FILE>      Parse one shortcut
      --code-page <CP>   ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file lnk -d evidence/ -o lnk.csv
juntei parse-file lnk -d evidence/ --code-page 932 -o lnk-ja.csv
juntei parse-file lnk -f suspicious.lnk -t jsonl | jq .
```

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| กลุ่มฟิลด์ | ฟิลด์และการตีความ |
|---|---|
| ต้นทาง | `SourceFile`, `SourceCreated`, `SourceModified`, `SourceAccessed`: พาธของทางลัดและเวลาในระบบไฟล์ของเครื่องที่พบ แยกจากเวลาของเป้าหมาย |
| เวลาของเป้าหมาย | `TargetCreated`, `TargetModified`, `TargetAccessed`: สแนปช็อตที่บันทึกไว้ใน LNK เป็นเบาะแสทางประวัติที่มีประโยชน์ ไม่ใช่เมทาดาทาปัจจุบันของเป้าหมาย |
| ข้อมูลพื้นฐานของเป้าหมาย | `FileSize`, `RelativePath`, `WorkingDirectory`, `FileAttributes`, `HeaderFlags`: บริบทของเป้าหมาย/การเรียกใช้ที่บันทึกไว้ อาร์กิวเมนต์สามารถเปลี่ยนความหมายได้ |
| วอลุ่ม | `DriveType`, `VolumeSerialNumber`, `VolumeLabel`: ระบุบริบทแบบ fixed/removable/network และใช้เชื่อมโยงกับทางลัดอื่น |
| พาธ | `LocalPath`, `NetworkPath`, `CommonPath`, `TargetIDAbsolutePath`: รูปแบบพาธที่เสริมกัน ให้ตรวจสอบกรณีที่ขัดแย้งกันและพาธใน shell namespace |
| พฤติกรรมการเรียกใช้ | `Args`, `ShowWindow`, `HotKey`, `ShimLayer`, `DarwinId`: วิธีที่ shell เรียกใช้หรือกำหนดค่าเป้าหมาย `Args` มีความสำคัญมาก |
| ตัวตนใน NTFS | `TargetMFTEntryNumber`, `TargetMFTSequenceNumber`: reference ของเป้าหมาย ณ เวลาที่สร้างลิงก์ ให้จับคู่ทั้งสองค่ากับ `$MFT` |
| การติดตาม | `MachineID`, `MachineMACAddress`, `TrackerCreatedOn`, `VolumeDroid`, `FileDroid`, `VolumeDroidBirth`, `FileDroidBirth`: distributed-link tracking ใช้เพื่อเชื่อมโยงข้อมูล ไม่ควรใช้ระบุตัวผู้กระทำเพียงลำพัง |
| บล็อกเพิ่มเติม | `ExtraBlocksPresent`: รายการโครงสร้างเสริมที่มีอยู่ ช่วยอธิบายว่าฟิลด์อื่นใดบ้างที่มีข้อมูลรองรับ |
| การแสดงผล | `Name`, `IconLocation`, `IconIndex`, `IconEnvironment`, `EnvironmentVariables`: ข้อมูลการแสดงผลและเป้าหมายที่ขยายตัวแปรสภาพแวดล้อมแล้ว ผู้โจมตีอาจใช้ชื่อ/ไอคอนที่ทำให้เข้าใจผิด |
| โฟลเดอร์ shell | `KnownFolder`, `SpecialFolder`: ตัวระบุโฟลเดอร์ shell ของ Windows ที่ถอดรหัสแล้ว |
| เครือข่าย | `NetworkDevice`, `NetworkProvider`: บริบทของแชร์/อุปกรณ์เครือข่ายและผู้ให้บริการ |
| การเข้ารหัสอักขระ | `ConsoleCodePage`: console code page ที่บันทึกไว้ (ถ้ามี) แยกจาก `--code-page` ของพาร์เซอร์ |
| ข้อมูลส่วนท้าย | `AppendedBytes`: ไบต์ที่อยู่หลังจุดสิ้นสุดเชิงตรรกะ ข้อมูลที่ไม่คาดคิดอาจบ่งชี้การลักลอบซ่อนข้อมูล (smuggling) หรือไฟล์ผิดรูปแบบ |
| การตรวจสอบความถูกต้อง | `Problems`: ความเสียหาย การถูกตัดทอน บล็อกที่ไม่สอดคล้องกัน หรือข้อจำกัดในการถอดรหัส ให้รวมไว้ในการตรวจทานด้วย |

JSON/JSONL จะเก็บรักษาข้อมูลเพิ่มเติม ได้แก่ target-ID item ทุกรายการ (ชนิด ชื่อ GUID เวลา FAT และ MFT
reference) extra-data block ทั้งหมด คุณสมบัติใน property store การตั้งค่า console และความสัมพันธ์เชิงโครงสร้างดิบ
ให้ใช้รูปแบบนี้กับทางลัดที่น่าสงสัย แทนที่จะพึ่งพาเฉพาะแถว CSV แบบแบน
เวลาใน `TargetItems` ที่ซ้อนอยู่มาจากนาฬิกา FAT ของ shell item ซึ่งไม่มีการจัดเก็บค่า offset
การแสดงผลแบบ `Z` ตามรูปแบบเดิมจึงไม่ได้พิสูจน์ว่าเป็น UTC โดย `auto-parse --timezone` จะคงค่าเหล่านี้ไว้ตามเดิม
ส่วน FILETIME ของเป้าหมายในส่วนหัว LNK และเวลาของ tracker ยังคงเป็น timestamp แบบสัมบูรณ์ที่แปลงเขตเวลาได้

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ตรวจสอบตำแหน่งต้นทางและเวลาของต้นทางก่อนตีความฟิลด์ของเป้าหมาย
2. สร้างคำสั่งเรียกใช้ที่มีผลจริงขึ้นใหม่จากพาธเป้าหมาย `Args`, `WorkingDirectory` และบล็อกตัวแปรสภาพแวดล้อม
3. ใช้ MFT reference ของเป้าหมาย serial ของวอลุ่ม พาธเครือข่าย droid, MachineID และ MAC เป็นจุดเชื่อมโยง (pivot) ข้ามหลักฐาน
4. ตรวจสอบ Prefetch และ event log สำหรับการเรียกใช้งาน และตรวจสอบ Jump Lists/RecentDocs สำหรับปฏิสัมพันธ์ของผู้ใช้
5. สำหรับไฟล์แนบที่เป็นอันตราย ให้เก็บรักษาข้อมูลที่ต่อท้ายไว้ และคำนวณแฮช/วิเคราะห์ตัวไฟล์ LNK เอง

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft Open Specifications: MS-SHLLINK](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-shllink/)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
- [Sootmark shell](https://github.com/Sootmark/shell)
- [LnkParse3](https://github.com/Matmaus/LnkParse3)
