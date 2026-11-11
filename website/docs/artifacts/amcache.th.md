# Amcache { #amcache }

## Amcache คืออะไร { #what-amcache-is }

Amcache คือบัญชีรายการด้านความเข้ากันได้ของแอปพลิเคชันของ Windows ที่จัดเก็บอยู่ในไฮฟ์รีจิสทรี ระบบรุ่นใหม่
บันทึกไฟล์ปฏิบัติการไว้ใต้ `InventoryApplicationFile` และอุปกรณ์ Plug-and-Play ไว้ใต้
`InventoryDevicePnp` ส่วน Windows รุ่นเก่าใช้โครงสร้างแบบเดิม `Root\File`

สำหรับการตรวจสอบไฟล์ปฏิบัติการ Amcache มีคุณค่าเพราะสามารถเก็บรักษาพาธเต็ม SHA-1 ขนาดไฟล์
เมทาดาทา PE ข้อมูลเวอร์ชัน ข้อความผู้เผยแพร่ (publisher) วันที่ลิงก์ และเวลาของรีจิสทรีไว้ได้
แม้ไฟล์ต้นฉบับจะหายไปแล้ว ระเบียนอุปกรณ์สามารถระบุฮาร์ดแวร์ที่เชื่อมต่อ ไดรเวอร์
วันที่ติดตั้ง และตัวระบุต่าง ๆ ได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Windows ได้ทำบัญชีรายการไฟล์หรืออุปกรณ์ที่บันทึกไว้
ระเบียนของไฟล์ปฏิบัติการเป็น **หลักฐานการมีอยู่ ไม่ใช่ข้อพิสูจน์การเรียกใช้งาน**

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\Windows\AppCompat\Programs\Amcache.hve
```

เก็บไฮฟ์พร้อมไฟล์ที่อยู่ข้างกัน ได้แก่ `Amcache.hve.LOG`, `.LOG1` และ `.LOG2` เมื่อไฮฟ์อยู่ในสถานะ dirty คำสั่ง `parse-registry amcache` จะเล่นซ้ำบันทึกธุรกรรมที่ใช้งานได้ในหน่วยความจำก่อนวิเคราะห์โดยอัตโนมัติ โดยไม่แก้ไขหลักฐาน รายละเอียดการกู้คืนจะแสดงในบันทึกของตัวแยกวิเคราะห์หรือเทอร์มินัล หากส่งระเบียนไปยัง stdout ข้อความวินิจฉัยจะส่งไปยัง stderr บันทึกที่หายไปหรือใช้ไม่ได้อาจทำให้ระเบียนไม่ครบหรืออ่านไฮฟ์ที่ไม่สอดคล้องกันไม่ได้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- รายการไฟล์ปฏิบัติการใน Amcache ไม่ได้พิสูจน์ว่าผู้ใช้เปิดใช้งานไฟล์นั้น
- `Publisher` คือเมทาดาทาจาก PE/version resource ที่ Windows บันทึกไว้ ไม่ได้พิสูจน์ว่าไฟล์มี
  ลายเซ็น Authenticode ที่ถูกต้อง ณ เวลาที่เก็บรวบรวม
- แฮชอาจไม่มีอยู่ และสคีมาแบบเดิม/แบบใหม่ไม่ได้ให้ค่าที่เหมือนกัน
- `LinkDate` มาจากส่วนหัว PE และอาจถูกปลอมแปลง คัดลอกมา หรือเกิดจาก reproducible build
- เวลาเขียนล่าสุดของคีย์อธิบายการอัปเดตระเบียนรีจิสทรี ไม่จำเป็นต้องเป็นเวลาดาวน์โหลด การสร้าง
  การเรียกใช้ครั้งแรก หรือการเรียกใช้ครั้งล่าสุด
- พาธอาจอ้างถึงวอลุ่มแบบถอดได้หรือวอลุ่มที่ไม่มีอยู่แล้ว

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry amcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Amcache hives
  -f, --file <FILE>      Parse one Amcache.hve

Records:
      --files    Include every inventoried executable
      --devices  Include every inventoried PnP device
  -a, --all      Include both (default for saved structured output)

Output:
  -o, --output <PATH>       Save text/JSON/JSONL or a directory of CSV tables
  -t, --output-type <TYPE>  csv, json, jsonl, or text
```

```bash
# Inspect the summary and findings
juntei parse-registry amcache -f Amcache.hve

# Save all record types as CSV tables
juntei parse-registry amcache -f Amcache.hve -a -o amcache-output

# Hunt for one SHA-1 in JSONL
juntei parse-registry amcache -f Amcache.hve -t jsonl \
  | jq 'select(.RecordType == "file" and .SHA1 == "0123456789abcdef0123456789abcdef01234567")'
```

เอาต์พุต CSV ประกอบด้วยตารางสรุป (summary) สิ่งที่พบ (findings) ไฟล์ (files) และอุปกรณ์ (devices) เมื่อใช้ `auto-parse` ตารางเหล่านี้คือ
`Registry-Amcache-Summary.csv`, `Registry-Amcache-Findings.csv`, `Registry-Amcache-Files.csv` และ
`Registry-Amcache-Devices.csv` ซึ่งอยู่ใต้ไดเรกทอรี `FileExistence/` ของแต่ละคอมพิวเตอร์โดยตรง โดยมี
`Registry-Amcache-logs.txt` อยู่ข้างตาราง หากมีหลายไฮฟ์จะเพิ่ม `-Source1`, `-Source2` เป็นต้น
ไว้ก่อนนามสกุลไฟล์ และ `OutputFiles` ใน manifest จะแสดงพาธที่แน่นอน การส่งออก CSV แบบเรียกใช้คำสั่งเดี่ยว
จะคงชื่อตารางเดิมและตำแหน่งเอาต์พุตที่ระบุไว้ ระเบียน JSONL ใช้ค่า `RecordType` ได้แก่
`summary`, `finding`, `file` และ `device`

## ฟิลด์เอาต์พุตของไฟล์ { #file-output-fields }

| ฟิลด์ | วิธีที่นักวิเคราะห์ควรใช้ |
|---|---|
| `KeyName`, `SourceKeyPath` | ระบุระเบียนรีจิสทรีต้นทางและสาขาของสคีมาที่แน่นอน เก็บรักษาไว้เพื่อให้ทำซ้ำผลได้ |
| `KeyLastWrittenUTC`, `KeyLastWrittenFiletime` | เวลาที่ระเบียนรีจิสทรีถูกอัปเดต ทั้งในรูปแบบที่อ่านได้และค่าดิบ อย่าระบุว่าเป็นเวลาการเรียกใช้งาน |
| `SourceVolumeKeyLastWrittenUTC`, `SourceVolumeKeyLastWrittenFiletime` | เวลาของคีย์วอลุ่มแม่ในสคีมาแบบเดิม เป็นข้อมูลบริบท ไม่ใช่เวลาที่เกิดการกระทำกับไฟล์ |
| `FileID` | ตัวระบุไฟล์ที่ Amcache บันทึกไว้ เก็บไว้ตามเดิมทุกประการ อย่าสันนิษฐานว่าเป็น SHA-1 เสมอ |
| `SHA1`, `SHA256` | จุดเชื่อมโยง (pivot) ที่แข็งแกร่งไปยัง Defender, threat intelligence, EDR และคลังไฟล์ ค่า null หมายถึงไม่ได้บันทึกไว้ |
| `Name`, `OriginalFileName`, `Description` | ชื่อจาก PE/version ที่บันทึกไว้ ให้เปรียบเทียบกับพาธจริงเพื่อตรวจจับการปลอมตัว (masquerading) เมทาดาทานี้ผู้โจมตีควบคุมได้ |
| `FullPath` | จุดเชื่อมโยงด้านตำแหน่งที่มีประโยชน์ที่สุด ให้ปรับรูปแบบตัวพิมพ์อย่างระมัดระวังแต่เก็บสตริงหลักฐานต้นฉบับไว้ |
| `LongPathHash` | ตัวระบุที่เกี่ยวกับพาธของ Windows ใช้จับคู่ระเบียนได้ ไม่ใช่แฮชของเนื้อหา |
| `ProgramID` | เชื่อมโยงไฟล์กับระเบียนบัญชีรายการแอปพลิเคชันเมื่อมีข้อมูล |
| `Publisher` | ข้อความผู้เผยแพร่ที่บันทึกไว้ ให้ถือเป็นเมทาดาทา ไม่ใช่การตรวจสอบลายเซ็น |
| `Version`, `BinFileVersion`, `ProductName`, `ProductVersion`, `BinProductVersion` | บริบทจาก version resource ความแตกต่างหรือค่าที่ไม่สมเหตุสมผลช่วยสนับสนุนการวิเคราะห์การปลอมตัวได้ |
| `BinaryType` | บริบทด้านสถาปัตยกรรม/ชนิดที่บันทึกไว้ ให้เปรียบเทียบกับการวิเคราะห์ PE เมื่อกู้คืนไฟล์ได้ |
| `LinkDate` | timestamp การคอมไพล์/ลิงก์ของ PE ตามที่บันทึกไว้ มีประโยชน์เฉพาะเมื่อใช้อย่างระมัดระวังและมีหลักฐานสนับสนุน |
| `Size` | ขนาดที่บันทึกไว้ ให้เปรียบเทียบกับ `$MFT` เนื้อหาใน Defender หรือไฟล์ที่กู้คืนได้ |
| `Language` | ตัวระบุภาษาของ PE เป็นเบาะแสเชิงบริบทที่อ่อน ไม่ใช่การระบุตัวผู้กระทำ |
| `IsPE-File`, `IsOS-Component` | แฟล็กการจำแนกประเภทที่ Windows บันทึกไว้ ค่า false ไม่ได้พิสูจน์ว่าไม่เป็นอันตราย |
| `USN` | ค่า update sequence ของ NTFS ที่ Amcache บันทึกไว้ ช่วยเชื่อมโยงไปยังระเบียน USN ที่ยังเหลืออยู่ |
| `APPX-PackageFullName`, `APPX-PackageRelativeID` | เชื่อมโยงแอปพลิเคชัน/คอมโพเนนต์แบบแพ็กเกจกับไฟล์ |
| `RegistryValues` | ค่ารีจิสทรีต้นฉบับทุกค่าพร้อมชนิด ค่าที่ถอดรหัสแล้ว และ hex ดิบ ใช้เมื่อไม่มีฟิลด์แบบมีชนิด หรือสำหรับฟิลด์ในอนาคต/แบบเดิม |

## ฟิลด์เอาต์พุตของอุปกรณ์ { #device-output-fields }

| กลุ่มฟิลด์ | ฟิลด์และการตีความ |
|---|---|
| ต้นทาง | `KeyName`, `SourceKeyPath`, `KeyLastWrittenUTC`, `KeyLastWrittenFiletime`: ระเบียนอุปกรณ์ที่แน่นอนและเวลาที่รีจิสทรีถูกอัปเดต |
| ตัวตน | `Description`, `BusDescription`, `Manufacturer`, `Model`: คำอธิบายอุปกรณ์สำหรับผู้ใช้ สตริงของผู้ผลิตอาจเป็นค่าทั่วไป |
| ตัวระบุฮาร์ดแวร์ | `HWID`, `CompID`, `Class`, `ClassGUID`, `ContainerID`, `DeviceInterfaceClasses`: จุดเชื่อมโยงที่แข็งแกร่งที่สุดสำหรับตระกูลอุปกรณ์และอินสแตนซ์ |
| สถานะ | `DeviceState`, `ProblemCode`, `ParentID`, `MatchingID`, `Enumerator`: บริบทการแจกแจงอุปกรณ์ (enumeration) และความสัมพันธ์ใน Windows |
| ไดรเวอร์ | `DriverID`, `DriverName`, `DriverPackageStrongName`, `DriverVerDate`, `DriverVerVersion`, `Provider`, `Service`, `INF`: ความเชื่อมโยงกับแพ็กเกจไดรเวอร์และบริการ ให้เชื่อมโยงกับไฟล์และบริการที่ติดตั้งไว้ |
| การติดตั้ง | `InstallDate`, `FirstInstallDate`, `InstallState`, `ExtendedInfs`: วงจรการติดตั้งที่บันทึกไว้ การไม่มีวันที่ไม่ได้พิสูจน์ว่าอุปกรณ์ไม่เคยถูกเชื่อมต่อ |
| ฟิลเตอร์และสแต็ก | `LowerClassFilters`, `UpperClassFilters`, `LowerFilters`, `UpperFilters`, `StackID`, `LocationPaths`: บริบทขั้นสูงของ driver stack ฟิลเตอร์ไดรเวอร์ที่ไม่คาดคิดควรได้รับการตรวจทาน |
| การเก็บรักษาค่าดิบ | `RegistryValues`: ค่าต้นทางครบถ้วนทั้งแบบมีชนิดและค่าดิบ สำหรับการตรวจสอบความถูกต้องและฟิลด์ที่ยังไม่รองรับ |

คอลัมน์อาร์เรย์และค่าดิบใน CSV ใช้คำต่อท้าย `JSON` (เช่น `HWID-JSON` และ
`RegistryValuesJSON`) เซลล์ของคอลัมน์เหล่านี้มี JSON ขณะที่ JSON/JSONL คงอาร์เรย์ตามรูปแบบเดิมไว้

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เชื่อมโยงข้อมูลโดยใช้ `FullPath`, `SHA1`, `Name` และ `USN`
2. ค้นหา Prefetch ที่ตรงกันเพื่อสนับสนุนการเรียกใช้งาน
3. ใช้ `$MFT` และบันทึก USN เพื่อยืนยันการสร้าง การเปลี่ยนชื่อ การลบ และขนาดของไฟล์
4. เปรียบเทียบเมทาดาทาผู้เผยแพร่/เวอร์ชันกับ PE ที่กู้คืนได้อย่างอิสระและผลการตรวจสอบลายเซ็น
5. ตรวจสอบข้อมูลที่ Defender กักกันและ event log เพื่อหาการตรวจจับหรือการลบออก

## รายละเอียดเพิ่มเติม { #further-details }

- [สคีมาเอาต์พุตของ ys-amcache-parser ของ Juntei](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
- [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser)
- [Security Ronin amcache-forensic](https://github.com/SecurityRonin/amcache-forensic)
- [ปลั๊กอิน Amcache ของ Regipy](https://github.com/mkorman90/regipy)
