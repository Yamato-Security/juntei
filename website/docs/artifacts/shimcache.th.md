# ShimCache (AppCompatCache) { #shimcache-appcompatcache }

## ShimCache คืออะไร { #what-shimcache-is }

ระบบย่อย Application Compatibility ของ Windows แคชพาธและเมทาดาทาของไฟล์ปฏิบัติการไว้ขณะประเมิน
พฤติกรรมด้านความเข้ากันได้/shim แคชนี้คงอยู่แม้ไฟล์จะถูกเปลี่ยนชื่อ ถูกลบ หรือถูกถอดออกพร้อม
วอลุ่มแบบถอดได้ โครงสร้างและฟิลด์ที่มีให้แตกต่างกันไปตั้งแต่ Windows XP จนถึง Windows 11

ShimCache มีคุณค่าในการค้นหาพาธในอดีต และในรูปแบบที่มีแฟล็กการเรียกใช้งานที่เชื่อถือได้
ก็ช่วยสนับสนุนการเรียกใช้งานเพิ่มเติมได้ เวลาที่แคชไว้โดยปกติคือ **เวลาแก้ไขล่าสุดของไฟล์**
ไม่ใช่เวลาที่ Windows เห็นหรือเรียกใช้ไฟล์นั้น

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Windows ได้สังเกตเห็นพาธนี้ในการประมวลผลด้าน
ความเข้ากันได้ รายการเพียงอย่างเดียวเป็นหลักฐานการมีอยู่/การสังเกตเห็น ให้ใช้ `Executed` เฉพาะเมื่อรูปแบบรองรับเท่านั้น

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

ไบต์อยู่ในไฮฟ์รีจิสทรี `SYSTEM` โดยทั่วไปอยู่ใต้:

```text
ControlSet00N\Control\Session Manager\AppCompatCache\AppCompatCache
```

ให้เก็บรวบรวม `SYSTEM`, `SYSTEM.LOG1` และ `SYSTEM.LOG2` Juntei อ่านทุก control set และระบุ
`Select\Current` เมื่อทำได้

!!! warning "รายการล่าสุดขณะระบบทำงานอาจไม่อยู่ในไฮฟ์"
    Windows อาจเก็บการอัปเดต ShimCache ไว้ในหน่วยความจำและเขียนลงดิสก์ระหว่างการปิดเครื่อง/รีบูต ไฮฟ์ที่คัดลอก
    จากระบบที่กำลังทำงานอาจไม่มีการสังเกตเห็นล่าสุด เวลาเขียนล่าสุดในรีจิสทรีของแคช
    ช่วยอธิบายว่าแคชถูกบันทึกเมื่อใด แต่ไม่ใช่เวลาการเรียกใช้งานของแต่ละรายการ

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- รายการในแคชเพียงอย่างเดียว **ไม่ใช่ข้อพิสูจน์การเรียกใช้งาน**
- `Executed` อาจเป็น `Yes`, `No` หรือ `NA` ความหมายที่ใช้ได้ขึ้นอยู่กับรูปแบบ/เวอร์ชัน
- `LastModifiedTimeUTC` เป็นของไฟล์ที่อ้างอิงถึง ไม่ใช่ของการสังเกตเห็นหรือการเรียกใช้งาน
- ลำดับมีประโยชน์แต่ไม่ใช่ไทม์ไลน์ที่แม่นยำในทุกกรณี รายการซ้ำเกิดขึ้นได้ข้าม control set/slack
- รายการ `FromSlack` ถูกกู้คืนจากนอกแคชที่ใช้งานอยู่ และต้องใช้ความระมัดระวังเพิ่มเติม
- `SYSTEM` ที่ dirty และไม่มี log ที่ใช้งานได้อาจขาดระเบียนปัจจุบัน ให้อ่านสรุปการกู้คืน
- สิ่งที่พบ (finding) บ่งชี้รูปแบบการตั้งชื่อ/พาธที่น่าสงสัย ไม่ใช่คำตัดสินว่าเป็นมัลแวร์

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry shimcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse SYSTEM or a raw AppCompatCache value
      --no-logs          Do not replay SYSTEM.LOG1/SYSTEM.LOG2

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry shimcache -f SYSTEM -o shimcache.csv
juntei parse-registry shimcache -d evidence/ -t jsonl \
  | jq 'select(.Executed == true or (.Findings | length > 0))'
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `ControlSet` | หมายเลข control set ที่เก็บแคช ให้เปรียบเทียบ `IsCurrent` ก่อนเลือกใช้ชุดใดชุดหนึ่ง |
| `CacheEntryPosition` | ตำแหน่ง/ลำดับภายในแคชที่ถอดรหัสแล้ว มีประโยชน์สำหรับลำดับสัมพัทธ์ ไม่ใช่เวลา |
| `Path` | พาธของไฟล์ปฏิบัติการ/แพ็กเกจที่แคชไว้ เป็นจุดเชื่อมโยงหลักในการค้นหา |
| `LastModifiedTimeUTC` | เวลาแก้ไขไฟล์ที่จัดเก็บใน entry ไม่ใช่เวลาการเรียกใช้งาน/การสังเกตเห็น |
| `Executed` | `Yes`, `No` หรือ `NA` ตามแฟล็กของรูปแบบที่รองรับ ใช้เฉพาะเมื่อมีบริบทของเวอร์ชัน |
| `Duplicate` | entry เดียวกันเคยพบแล้วใน control set หรือ slack ก่อนหน้า ช่วยป้องกันการนับซ้ำ |
| `SourceFile` | ไฮฟ์/ค่าดิบที่เป็นอินพุต จำเป็นเมื่อทำงานข้ามโฮสต์ |
| `FromSlack` | entry ที่กู้คืนจาก slack ของค่าแคช แทนที่จะเป็นรายการที่ใช้งานอยู่ ความเชื่อมั่นต่ำกว่าและอาจเก่ากว่า |
| `CacheFormat` | โครงสร้าง Windows/แคชที่ถอดรหัสแล้ว เป็นตัวกำหนดฟิลด์ที่มีและความหมาย |
| `CacheLastWrittenUTC` | เวลาที่คีย์รีจิสทรี/แคชถูกบันทึก เป็นบริบทร่วม ไม่ใช่เวลาที่แต่ละ entry ทำงาน |
| `InsertFlags`, `ShimFlags` | แฟล็กดิบด้านความเข้ากันได้ `Executed` คือผลที่ Juntei ตีความในกรณีที่รองรับ |
| `DataSize` | ขนาดข้อมูลของ entry นี้ มีประโยชน์สำหรับการตรวจสอบเชิงโครงสร้าง |
| `Machine` | สถาปัตยกรรมที่รูปแบบใหม่บันทึกไว้ (`x64`, `x86`, `ARM64`) |
| `FileSize`, `LastUpdateTimeUTC` | เมทาดาทาที่แคชไว้เฉพาะของ Windows XP เมื่อมีอยู่ |
| `PackageName`, `PackageVersion`, `PackagePublisherID` | ตัวตนของแอปพลิเคชันแบบแพ็กเกจใน entry ที่รองรับ Publisher ID เป็นบริบท ไม่ใช่ข้อพิสูจน์ลายเซ็น |
| `Findings` | รหัสความผิดปกติของ Juntei เช่น ไบนารีระบบที่ถูกย้ายตำแหน่งหรือพาธ staging ที่น่าสงสัย เป็นเพียงเบาะแส |
| `RegistryPath`, `RegistryValueName` | แหล่งคีย์/ค่าในไฮฟ์ที่แน่นอน เก็บรักษาไว้เพื่อการตรวจสอบ |
| `CacheValueSize` | ขนาดของค่ารีจิสทรีทั้งหมด ไม่ใช่ `DataSize` ของ entry แต่ละรายการ |
| `IsCurrent` | control set ของ entry นี้ตรงกับ `SYSTEM\Select\Current` หรือไม่ ว่างเปล่าหากไม่สามารถทราบได้อย่างแน่ชัด |
| `SelectCurrent`, `SelectDefault`, `SelectLastKnownGood`, `SelectFailed` | หมายเลข control set ดิบจาก `SYSTEM\Select` สำหรับบริบทและการวิเคราะห์การกู้คืน |

JSON/JSONL ยังเก็บรักษาข้อมูล entry, tag, ระเบียนย่อยของแอปแบบแพ็กเกจ, finding แบบมีโครงสร้าง,
ปัญหาของแคช และรายละเอียดการกู้คืนเพิ่มเติมด้วย

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ให้ความสำคัญกับ control set ปัจจุบัน แต่เก็บชุดอื่นและ slack ไว้เป็นหลักฐานในอดีต
2. ค้นหาพาธและ finding อย่าใช้ `LastModifiedTimeUTC` เป็นเวลาการเรียกใช้งาน
3. เมื่อ `Executed == Yes` ให้ยืนยันด้วย Prefetch, SRUM, event log หรือร่องรอยขณะทำงานอื่น ๆ
4. จับคู่พาธกับ `$MFT`/USN เพื่อระบุประวัติการสร้าง การลบ และเมทาดาทา
5. รายงานว่ามีการเล่นซ้ำ transaction log หรือไม่ และแคชขณะระบบทำงานอาจยังไม่ได้ถูกเขียนลงดิสก์หรือไม่

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser)
- [Security Ronin shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic)
- [Mandiant: การใช้ประโยชน์จาก Application Compatibility Cache ในการสืบสวนทางนิติวิทยาศาสตร์](https://www.mandiant.com/resources/blog/leveraging-application-compatibility-cache-forensic-investigations)
- [ข้อกำหนดรูปแบบรีจิสทรีของ Windows โดย Maxim Suhanov](https://github.com/msuhanov/regf)
