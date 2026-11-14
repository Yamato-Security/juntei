# วิเคราะห์ข้อมูลที่รวบรวมโดยอัตโนมัติ { #automatic-collection-parsing }

`auto-parse` เป็นวิธีที่เร็วที่สุดในการแปลงชุดข้อมูล triage ของ Windows ที่เก็บรวบรวมจากคอมพิวเตอร์หลายเครื่องให้เป็น
ไดเรกทอรีการวิเคราะห์ที่มีโครงสร้าง เพียงระบุรากของหลักฐานและไดเรกทอรีเอาต์พุตใหม่ คำสั่งนี้จะค้นหา
ระบบที่ถูกเก็บรวบรวม ระบุชื่อคอมพิวเตอร์ และเรียกใช้พาร์เซอร์ของ Juntei ทุกตัวที่ใช้ได้
แบบขนาน

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT>
```

เอาต์พุตเริ่มต้นเป็น CSV สามารถเพิ่มไทม์ไลน์ JSONL สำหรับ `jq` หรือการนำเข้า SIEM โดยยังคงผลลัพธ์ CSV
เหล่านั้นไว้ได้ดังนี้

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> -j <OUTPUT>/timeline.jsonl
```

!!! warning "แยกโครงสร้างไดเรกทอรีเอาต์พุตและหลักฐานออกจากกัน"
    Juntei จะปฏิเสธไดเรกทอรีเอาต์พุตที่อยู่ภายใน `<EVIDENCE>` หรือมี `<EVIDENCE>` อยู่ภายใน ซึ่ง
    ป้องกันไม่ให้ข้อมูลที่สร้างขึ้นถูกค้นพบซ้ำเป็นหลักฐาน และป้องกันไม่ให้ `--clobber`
    แตะต้องไฟล์ในชุดข้อมูลต้นทาง

## อิมเมจดิสก์ VHDX { #vhdx-disk-images }

เมื่อใช้ `--vhdx` คำสั่ง `auto-parse` อ่าน VHDX แบบเนทีฟบน **Linux, macOS และ Windows** ผ่าน [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) โดยไม่ต้องเมานต์ OS ใช้ FUSE หรือมีสิทธิ์ผู้ดูแลระบบ

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

พาร์เซอร์ต้องใช้พาธ Juntei จึงคัดลอกไฟล์ที่พาร์เซอร์อ่านไปยังพื้นที่ชั่วคราวส่วนตัวด้วยบัฟเฟอร์จำกัด ได้แก่ ไฮฟ์รีจิสทรีและล็อกธุรกรรม ไฟล์เมทาดาทา NTFS ฐานข้อมูล ไฟล์ Prefetch ช็อตคัต Jump List ไฟล์ Outlook ไฟล์ดัชนีของถังรีไซเคิล และโฟลเดอร์กักกันของ Defender ไฟล์อื่นทั้งหมดจะเป็นไฟล์แทนที่ว่างที่คงเวลาแก้ไขเดิมไว้ ไดเรกทอรีและพาธจึงดูเหมือนบนดิสก์ persistence-hunter จะอ่านเนื้อหาของไฟล์แทนที่จากอิมเมจเฉพาะเมื่อจำเป็น เช่น ไฟล์ปฏิบัติการที่รายการ persistence อ้างถึง **ต้องมีพื้นที่สำหรับไฟล์ที่คัดลอก** นอกเหนือจากผลลัพธ์ หากรองรับจะเขียนพื้นที่ศูนย์เป็นพื้นที่ sparse โดยรักษาชื่อ โครงสร้าง ฮาร์ดลิงก์ และเวลาแก้ไข เวลาแหล่งข้อมูล LNK และ Jump List มาจากสแนปช็อตเมทาดาทา NTFS เดิม ไม่ใช่สำเนาชั่วคราว เมื่อจบตามปกติรวมถึงกรณีพาร์เซอร์ผิดพลาดจะลบสำเนา แต่การบังคับหยุดอาจทิ้งไฟล์ไว้ ต้นฉบับอ่านอย่างเดียวและใช้ล็อก VHDX ที่ค้างในหน่วยความจำเท่านั้น

การค้นหาโฮสต์ CSV/JSON/JSONL และ `-j` ใช้ขั้นตอนปกติ `Image` ใน `manifest.json` ระบุอิมเมจต้นฉบับ ส่วน `EvidenceRoot` ระบุพาธสัมพัทธ์ภายใน ข้อผิดพลาดการอ่านและชื่อไม่ปลอดภัยอยู่ใน `ImageErrors` และแถวล้มเหลวของ `manifest.csv` จะเก็บผลที่อ่านได้และคืนรหัส 1 หากอินพุตไม่สมบูรณ์ ไม่เปลี่ยนชื่ออันตรายหรือกำกวม และไม่แยกสตรีม NTFS ที่มีชื่อเป็นไฟล์เพิ่มเติม อาร์ติแฟกต์ที่รวบรวมเป็นไฟล์ปกติยังใช้ได้

คำสั่ง [`mount` แยกต่างหาก](mount.md) เปิดระบบไฟล์ให้โปรแกรมอื่นและยังต้องใช้ Linux FUSE โดย `auto-parse --vhdx` ไม่ได้เรียกใช้คำสั่งนี้

พื้นที่เตรียมข้อมูลใช้ไดเรกทอรีส่วนตัวบนระบบไฟล์ของผลลัพธ์เป็นค่าเริ่มต้น ใช้ `--temp-dir` เพื่อเลือกไดเรกทอรีที่มีอยู่บนดิสก์อื่น เมทาดาทาต้นฉบับเก็บแยกบนระบบไฟล์ผลลัพธ์ จึงยังแยกวิเคราะห์ไฟล์ที่อ่านได้แล้วเมื่อดิสก์เตรียมข้อมูลเต็ม ควรสำรองพื้นที่สำหรับทั้งสำเนาและผลลัพธ์ หากวอลุ่มผลลัพธ์เต็มจะบันทึกผลไม่ได้ Ctrl+C หยุดโปรเซสตัวแยกวิเคราะห์ ล้างพื้นที่ส่วนตัว และออกด้วยรหัส 130 แต่ SIGKILL หรือไฟดับยังอาจทิ้งสำเนาไว้ การค้นหาอิมเมจจะข้ามไดเรกทอรีย่อยที่อ่านไม่ได้ แต่รากอินพุตที่อ่านไม่ได้ยังเป็นข้อผิดพลาด

ไดเรกทอรีเตรียมข้อมูลต้องอยู่นอกไดเรกทอรีผลลัพธ์ บน Linux และ macOS สัญญาณ SIGTERM และ SIGHUP จะหยุดโปรเซสตัวแยกวิเคราะห์และล้างพื้นที่ทำงานด้วย

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## อิมเมจดิสก์ E01 และ Ex01 { #e01-and-ex01-disk-images }

`--e01` อ่านอิมเมจแบบกายภาพ EWF1 (`.E01`) และ EWF2 (`.Ex01`) แบบเนทีฟบน **Linux, macOS และ Windows** โดยไม่ต้องใช้ FUSE การเมานต์ของ OS หรือสิทธิ์ผู้ดูแลระบบ ใช้ [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser) และกระบวนการแยกวิเคราะห์ NTFS เดียวกับที่อธิบายไว้ข้างต้น

สำหรับอิมเมจที่แบ่งส่วน ให้ระบุส่วนแรก เก็บไฟล์ประกอบทั้งหมดรวมถึง `.E02` ไว้ข้างกันด้วยชื่อเดิม รองรับทั้งดิสก์เต็ม (MBR/GPT) และวอลุ่ม NTFS เดี่ยว ใช้ `--vhdx --e01` สำหรับไดเรกทอรีที่มีทั้งสองรูปแบบ

ข้อกำหนดพื้นที่ชั่วคราวและเมทาดาทาต้นฉบับเหมือนกัน อิมเมจ E01 มักเก็บทั้งดิสก์ และอิมเมจทั้งหมดจะถูกเตรียมข้อมูลก่อนเริ่มแยกวิเคราะห์ แต่จะคัดลอกเฉพาะไฟล์ที่พาร์เซอร์อ่าน ซึ่งมักเป็นส่วนเล็ก ๆ ของพื้นที่ที่ใช้งาน อิมเมจเปิดแบบอ่านอย่างเดียว ช่องว่างจากการเก็บข้อมูลและความเสียหายที่พบขณะเปิดหรืออ่านจะแสดงใน `ImageErrors` และ `manifest.csv` โดยเก็บผลลัพธ์ที่อ่านได้และจบด้วยรหัส 1 ตรวจเฉพาะข้อมูลที่เข้าถึง ไม่ใช่ทุกชังก์หรือแฮชของสื่อทั้งหมด ปฏิเสธไฟล์เชิงตรรกะ `.L01` และ `.Lx01` และไม่มีตัวเลือกรหัสผ่าน

คำสั่ง `mount` ที่แยกต่างหากยังต้องใช้ Linux FUSE แต่การแยกวิเคราะห์อัตโนมัติแบบเนทีฟไม่ต้องใช้

```bash
juntei auto-parse --e01 -d evidence -o parsed
juntei auto-parse --e01 -d disk.E01 -o parsed
juntei auto-parse --vhdx --e01 -d evidence -o parsed -j timeline.jsonl
```

## เขตเวลาของเอาต์พุต { #output-time-zone }

timestamp ที่พาร์สแล้วใช้ UTC เป็นค่าเริ่มต้น โดยไม่ขึ้นกับเขตเวลาท้องถิ่นของคอมพิวเตอร์ของนักวิเคราะห์ UTC
ใช้รูปแบบของ Windows event log `2026-10-20T08:46:38.2001668Z` ได้แก่ วันที่/เวลาแบบ ISO 8601 มี `T`
คั่นระหว่างวันที่และเวลา และ `Z` สำหรับ UTC ตัวเลขทศนิยมที่มีอยู่จะถูกเก็บรักษาไว้
`GeneratedUTC` ของ `manifest.json` ก็ใช้รูปแบบนี้เช่นกัน ส่วน `Start time` และ `End time` บนคอนโซล
เป็นเวลาท้องถิ่นของคอมพิวเตอร์ของนักวิเคราะห์ เช่น `2026-11-04 09:15:02` ไม่ว่าเขตเวลาของเอาต์พุต
จะเป็นอย่างไร

เลือก offset การแสดงผลแบบคงที่ด้วย `-Z, --timezone`:

```bash
juntei auto-parse -d evidence/ -o parsed-jst/ --timezone "+09:00"
juntei auto-parse -d evidence/ -o parsed-offset/ -Z "-01:00"
juntei auto-parse -d evidence/ -o parsed-half-hour/ -Z "+02:30"
```

offset คือค่า `HH:MM` แบบมีเครื่องหมายและเป็นตัวเลขสองหลัก (ชั่วโมง `00–23` นาที `00–59`) ชื่อเช่น
`JST` ค่าแบบย่อเช่น `+9` และช่วงค่าที่ไม่ถูกต้องจะทำให้เกิดข้อผิดพลาดพร้อมตัวอย่างไวยากรณ์
ให้ละตัวเลือกนี้หรือใช้ `--timezone "+00:00"` สำหรับ UTC offset แบบคงที่ไม่ได้ใช้กฎเวลา
ออมแสง

ตัวเลือกนี้มีผลกับ timestamp ที่พาร์สแล้วในผลลัพธ์ CSV, JSON และ JSONL ของแต่ละอาร์ติแฟกต์ และ
การส่งออก JSONL เพิ่มเติมด้วย `-j` ตัวอย่างเช่น `2026-10-20T08:46:38.2001668Z` จะกลายเป็น
`2026-10-20T17:46:38.2001668+09:00` ซึ่งทั้งสองค่าอธิบายช่วงเวลาเดียวกัน

ชื่อฟิลด์ยังคงเดิม รวมถึงชื่อที่ลงท้ายด้วย `UTC` ซึ่งมีอยู่แล้ว ค่าที่แปลงแล้วจะระบุ
offset ที่เลือกไว้อย่างชัดเจน `OutputTimeZone` ใน manifest ทั้งสองจะบันทึก offset การแสดงผลที่ร้องขอ
ขณะที่ `GeneratedUTC` ยังคงเป็นเมทาดาทาการทำงานแบบ UTC บันทึกของแต่ละพาร์เซอร์จะเก็บข้อมูลสรุปแบบ UTC ดั้งเดิมของพาร์เซอร์ไว้

ตัวเลข FILETIME/epoch ดิบ ดิกชันนารีค่ารีจิสทรี การกำหนดค่าเขตเวลาของต้นทาง เมทาดาทา
แบบข้อความ ค่าที่มีเฉพาะวันที่ และค่านาฬิกาท้องถิ่นที่ไม่ทราบ offset จะถูกเก็บรักษาไว้ตามเดิม Juntei
จะไม่เติม `Z` ให้กับเวลาท้องถิ่นที่ไม่ทราบ offset นาฬิกา shell item ที่มาจาก FAT (`Item*UTC` ใน
ShellBags/OpenSaveMRU และ `TargetItems` ของ LNK/Jump List) ไม่มี offset ที่บันทึกไว้ ป้ายชื่อ
UTC แบบดั้งเดิมหรือคำต่อท้าย `Z` ที่แสดงผลไม่ได้พิสูจน์ว่าเป็น UTC ตัวเลือก offset จึงเก็บค่าเหล่านี้ไว้ตามเดิม
คอลัมน์ CSV ที่มีข้อมูลสรุปแบบข้อความอิสระจะเก็บ
ข้อความตามที่บันทึกไว้ ส่วน JSON/JSONL จะเก็บ timestamp ที่พาร์สแล้วแบบซ้อนกันไว้เป็นฟิลด์ที่มีโครงสร้าง

UTC ใช้เอาต์พุตของพาร์เซอร์ที่มีอยู่โดยตรง offset ที่ไม่ใช่ศูนย์ที่ระบุอย่างชัดเจนจะเพิ่มขั้นตอนการจัดรูปแบบ
แบบสตรีมภายใน worker ของแต่ละพาร์เซอร์ ซึ่งเปลี่ยนค่าที่แสดงผลโดยไม่เปลี่ยนหลักฐาน

หาก offset จะทำให้ timestamp เลื่อนออกนอกช่วงปี `0000–9999` ค่านั้นจะคง offset
ดั้งเดิมที่ระบุไว้ ค่าที่มีทศนิยมเกินเก้าหลักก็จะคงเดิมเช่นกันแทนที่จะ
ถูกปัดเศษ Juntei จะบันทึกคำเตือนไว้ในไฟล์ `*-logs.txt` ของพาร์เซอร์และแปลง
timestamp อื่น ๆ ต่อไป CSV/JSON ที่ผิดรูปแบบหรือข้อผิดพลาดของเอาต์พุตยังคงทำให้งานล้มเหลว และไฟล์ผลลัพธ์
ดั้งเดิมที่ได้รับผลกระทบจะไม่ถูกเปลี่ยนแปลง

## สิ่งที่คำสั่งนี้ค้นพบ { #what-it-discovers }

Juntei ค้นหาแบบเรียกซ้ำสำหรับไดเรกทอรีชื่อ `Windows` ที่มีไดเรกทอรี `System32`, `Prefetch`
หรือ `AppCompat` อยู่ภายใน โดยไม่สนใจตัวพิมพ์เล็กใหญ่ การรู้จักสองชื่อหลังยังรองรับการเก็บรวบรวม
แบบเจาะจงที่มีเพียง Prefetch หรือ Amcache ด้วย วิธีนี้จะไม่รวมไดเรกทอรีโปรไฟล์ผู้ใช้ เช่น
`AppData\Roaming\Microsoft\Windows` การติดตั้งแบบเก่าที่ซ้อนอยู่ เช่น
`C:\Windows.old\Windows` จะถูกกำหนดให้เป็นส่วนหนึ่งของวอลุ่มที่เก็บรวบรวมด้านนอกแทนที่จะกลายเป็นคอมพิวเตอร์
ที่ซ้ำกัน ความลึกสูงสุดเริ่มต้นคือ 10 ซึ่งครอบคลุมพาธทั่วไปของ Velociraptor เช่น

```text
<EVIDENCE>/<collection>/uploads/auto/C%3A/Windows
```

ให้เปลี่ยนขีดจำกัดเฉพาะเมื่อชุดข้อมูลซ้อนกันลึกกว่านั้น:

```bash
juntei auto-parse -d evidence -o parsed --max-depth 15
```

สำหรับแต่ละระบบที่ค้นพบ ชื่อคอมพิวเตอร์จะถูกเลือกตามลำดับต่อไปนี้

1. `hostname` ใน `client_info.json` ของไดเรกทอรีบรรพบุรุษที่ใกล้ที่สุด (โดยเก็บค่าไว้เป็น `FQDN` ใน manifest)
2. `ComputerName` ในไฮฟ์รีจิสทรี `SYSTEM` ที่เก็บรวบรวมมา
3. ชื่อไดเรกทอรีของชุดข้อมูลที่ใกล้ที่สุดซึ่งใช้ได้ โดยข้ามไดรฟ์และโครงสร้าง `uploads/auto` ของ Velociraptor
   และตัดคำต่อท้าย เช่น `_F.…` ออก

ชื่อจะถูกปรับให้ปลอดภัยสำหรับใช้เป็นชื่อไดเรกทอรี ชื่อที่ซ้ำกันจะได้รับคำต่อท้ายเป็นตัวเลขทั้งใน
ไดเรกทอรีเอาต์พุตและค่า `Comp` เพื่อให้ยังแยกแยะระบบต่าง ๆ ได้ใน JSON
manifest จะเก็บชื่อโฮสต์ตามที่ค้นพบทุกประการไว้ในฟิลด์ `Hostname` แยกต่างหาก

หลังจากแสดงรายการระบบแล้ว `auto-parse` จะพิมพ์บรรทัด `Timezone:` หนึ่งบรรทัดไว้เหนือ
`Parser tasks` โดยตรง ซึ่งแสดงเขตเวลาเอาต์พุตที่เลือก: `UTC` โดยค่าเริ่มต้น หรือค่าเช่น
`UTC+09:00` เมื่อใช้ `--timezone "+09:00"` ป้ายชื่อเป็นสีเขียวและค่าใช้สีข้อความเริ่มต้นของ
เทอร์มินัล เช่นเดียวกับค่าสรุปอื่น ๆ `-K, --no-color` จะแสดงบรรทัดนี้แบบไม่มีสี
การกำหนดค่าเขตเวลาของระบบที่เก็บรวบรวมมาดูได้จากผลลัพธ์
[ข้อมูลระบบ](../artifacts/system-info.md)

## พาร์เซอร์ที่เรียกใช้ { #parsers-it-runs }

พาร์เซอร์หนึ่งตัวที่ล้มเหลวจะไม่หยุดพาร์เซอร์ตัวอื่น `auto-parse` เรียกใช้งานเหล่านี้กับคอมพิวเตอร์ทุกเครื่อง
ที่ค้นพบ งานแต่ละงานใช้คำสั่งที่จัดกลุ่มตามแหล่งที่มาเดียวกับการพาร์สด้วยตนเอง เช่น
`parse-ntfs mft`, `parse-registry amcache`, `parse-ese srum` และ `parse-file prefetch`:

- Amcache
- `$MFT`
- เรคคอร์ด redo/undo ของ NTFS ใน `$LogFile`
- เรคคอร์ด Prefetch และไทม์ไลน์การเรียกใช้งานจาก Prefetch แยกต่างหาก
- USN Journal
- ดัชนีไดเรกทอรี `$I30`
- บูตเซกเตอร์ `$Boot` และตัวบอกความปลอดภัย `$Secure:$SDS`
- SRUM
- การบันทึกการเข้าถึงของผู้ใช้ (UAL)
- ประวัติ การดาวน์โหลด คุกกี้ และประวัติฟอร์มของเว็บเบราว์เซอร์ (เบราว์เซอร์ที่ใช้ Chromium และ Firefox เป็นพื้นฐาน)
- WebCache ของ Internet Explorer และ Edge รุ่นเก่า: ประวัติ การดาวน์โหลด คุกกี้ และรายการแคช
- ไฟล์ PST, OST, MSG และ OFT ของ Outlook: โฟลเดอร์ รายการ ผู้รับ สิ่งที่แนบ ผู้ติดต่อ รายการปฏิทิน
  และงาน
- ทางลัด LNK
- Jump Lists
- ถังรีไซเคิล: พาธเดิม ขนาด เวลาที่ลบ และผู้ใช้ของรายการที่ถูกลบ
- ShimCache
- Persistence Hunter (การตรวจสอบการคงอยู่ข้ามแหล่งข้อมูล 131 รายการพร้อมโปรไฟล์ allow/block)
- เมทาดาทาการกักกันของ Defender
- ประวัติของกล่องโต้ตอบ Open และ Save As
- คีย์และค่ารีจิสทรีทั้งหมด รวมถึงข้อมูลที่ถูกลบซึ่งกู้คืนได้
- บริการ, RecentDocs, TaskCache, บัญชี SAM, ตำแหน่งการเริ่มอัตโนมัติผ่านรีจิสทรี, UserAssist, BAM,
  โปรแกรมที่ติดตั้ง, MuiCache, ShellBags และอุปกรณ์ USB
- ข้อมูลระบบ: ชื่อ เวอร์ชัน Windows เขตเวลา การปิดเครื่องครั้งล่าสุด โปรไฟล์ และ SID
- นโยบายการตรวจสอบ (audit policy): เหตุการณ์ด้านความปลอดภัยใดบ้างที่ระบบถูกตั้งค่าให้บันทึก
- การกำหนดค่าบูต (BCD และ BCD-Template)
- โปรไฟล์เครือข่าย อินเทอร์เฟซ แชร์ port proxy และเส้นทาง
- ชื่อและเวลาของ LSA secrets และการเข้าสู่ระบบโดเมนที่แคชไว้ซึ่งใช้งานอยู่ (ไม่มีการถอดรหัสใด ๆ)

Persistence Hunter เก็บผลลัพธ์ทุกระดับความรุนแรง (`--min-severity info`) รวมถึงรายการที่ตรงกับ allow เพียงบางส่วน
`LOW` เป็นการจัดประเภทที่ควรตรวจสอบ การไม่มีเมทาดาทาลายเซ็นก็อาจทำให้คำสั่งที่เป็นอันตราย
ได้ระดับนี้เช่นกัน การรวบรวมแบบอัตโนมัติจะไม่ตัดรายการเหล่านี้ออกโดยใช้เกณฑ์ medium
เริ่มต้นของคำสั่งที่เรียกใช้ด้วยตนเอง

hunter ใช้ worker หนึ่งตัวภายในโพรเซสพาร์เซอร์อัตโนมัติแต่ละโพรเซส ซึ่งป้องกันไม่ให้ `auto-parse -T <N>`
คูณงบประมาณโพรเซสด้วย pool ขนาดเท่าจำนวน CPU อีกชุดในทุกโพรเซสลูก ส่วน
`persistence-hunter -T <N>` แบบเรียกใช้เดี่ยวจะควบคุมจำนวนเธรด worker ของ hunter เอง

เอาต์พุต `$MFT` แบบ CSV มี timestamp และชื่อสั้นทั้งหมด เอาต์พุตรีจิสทรีมีข้อมูลดิบและ
การตีความค่า `REG_QWORD` เป็น FILETIME เอาต์พุต SRUM แบบ JSON และ JSONL มีเมทาดาทา
สถานะของค่า การดึงไฟล์จาก Defender quarantine ยังคงต้องเปิดใช้เอง

เมื่อใช้ `--mft-slack` เอาต์พุต `$MFT` จะรวมชื่อไฟล์ที่หลงเหลืออยู่ใน slack ของระเบียน FILE ด้วย ในรูปแบบเดียวกับที่ [`parse-ntfs mft -x`](../artifacts/mft.md) เขียน: แถวที่ทำเครื่องหมาย `FromSlack` ใน CSV และ `SlackFileNames` ใน JSON และ JSONL ส่วน `--code-page` ใช้กับไฟล์ LNK, Jump List, ShellBags และ OpenSaveMRU

อาร์ติแฟกต์ที่ไม่ได้ถูกเก็บรวบรวมจะเป็นงานที่ `skipped` ตามปกติ ไม่ใช่ความล้มเหลว


## การสร้างระบบไฟล์ขึ้นใหม่ { #filesystem-reconstruction }

MFT ที่เก็บมาทุกไฟล์จะสร้างต้นไม้โฟลเดอร์ที่เปิดดูได้เพิ่มด้วย ตารางเดิมไม่เปลี่ยนในโหมด CSV, JSON และ JSONL

สร้างต้นไม้ในพื้นที่ส่วนตัวข้างปลายทางและเผยแพร่เมื่อเสร็จเท่านั้น ข้อผิดพลาดหรือการขัดจังหวะที่จัดการได้จะล้างพื้นที่และเก็บต้นไม้เดิม การบังคับปิดอาจเหลือโฟลเดอร์ `.juntei-reconstruction-*` แต่ไม่ขวางการลองใหม่หรือกลายเป็นโวลุ่มปัจจุบัน

อักษรไดรฟ์ที่อนุมานจากพาธที่เก็บจะใช้โฟลเดอร์ เช่น `FileSystem/C` ตัว MFT ไม่เก็บอักษรไดรฟ์ หากระบุไม่ได้ใช้ `UnknownVolume` และแยก MFT หลายไฟล์ เช่น `C-MFT-2`

ไฟล์และโฟลเดอร์ที่ไม่ได้จัดสรรจะต่อท้ายด้วย `-deleted` รวมถึงโฟลเดอร์แม่ที่ลบแล้วในพาธลูก กู้ DATA แบบ resident ไม่มีชื่อที่ยังเหลืออยู่ ส่วน non-resident หรือเนื้อหาที่อ่านไม่ได้สร้างเป็นไฟล์ว่าง ขนาดเชิงตรรกะไม่ใช่จำนวนไบต์ที่กู้ได้

`-i30` ระบุชื่อดัชนีเพิ่มเติมที่ยังยืนยันการลบไม่ได้ ใช้ดัชนี resident ใน MFT และ I30 ที่เก็บจากโวลุ่มเดียวกัน Slack อาจเหลือชื่อก่อนเปลี่ยนหรือสำเนาจากการปรับสมดุล ไม่ดึงเนื้อหาจากไฟล์คนละรุ่น

`_Juntei-Unresolved` เก็บพาธที่แม่หาย ถูกใช้ซ้ำ วนเป็นวง หรืออยู่ลึกเกินไป ชื่อไม่ปลอดภัยจะ escape และเก็บชื่อชนกันแยกกัน ชื่อเดิมยังอยู่ในรายการ ละชื่อ DOS เมื่อมีชื่อยาว และสร้าง hard link เป็นสำเนาอิสระ

`_Juntei-Reconstruction.csv` บันทึกการสร้างใหม่: `DataStatus`, `LogicalSize`, `RecoveredBytes`, `OriginalPath`, `OutputPath` แหล่งที่มา: `EntryNum`, `SN`, `ParentEntryNum`, `ParentSN`, `Source`, `FromSlack`, `Offset` รวมไดรฟ์ที่อนุมาน สถานะลบ และชนิดโฟลเดอร์ รายการที่ยังไม่เสร็จลงท้าย .partial.csv และใช้ชื่อสุดท้ายเมื่อสำเร็จ

`manifest.json` / `manifest.csv`: `ReconstructionRoot`, `ReconstructionInventory`, `ReconstructionStatus`, `ReconstructionError`.

อินพุต I30 เสริมที่อ่านไม่ได้หรือเสียหายจะบันทึกเป็น `i30_input_error` โดยยังเก็บผล MFT ที่ถูกต้องไว้

ไฟล์ที่มีเนื้อหา resident ที่กู้ได้และไม่ว่างจะเติม `.dontrun` ก่อนเขียนไบต์ ชื่อเดิมอยู่ในรายการ ส่วนไฟล์ว่างคงชื่อเดิม ห้ามรันเนื้อหาที่กู้ได้ ไม่คืนค่า reparse point, ACL, เวลาเดิม หรือสิทธิ์รัน สตรีมที่มีชื่อต้องแยกออกต่างหาก

แม้ไฟล์ว่างก็ใช้ inode และเมทาดาทาโฟลเดอร์ หากไม่ต้องการให้ข้ามการสร้างใหม่:

```bash
juntei auto-parse -d <EVIDENCE> -o <OUTPUT> --no-reconstruct-filesystem
```

การเขียนทับจะแทนที่เฉพาะต้นไม้ที่มีรายการครบและไม่มีสิ่งที่หาย เพิ่ม หรือลิงก์ เก็บต้นไม้บางส่วนและไฟล์เพิ่ม เช่น `.DS_Store` ไว้ Auto-parse ข้ามเฉพาะการสร้างใหม่ บันทึกโฟลเดอร์และเหตุผลใน log กับ manifest ทั้งสองและทำ parser อื่นต่อ ข้อผิดพลาดเขียนรายงานแยกและคืนสถานะล้มเหลวโดยเก็บผลที่แยกไว้ คำสั่งเดี่ยวยังปฏิเสธปลายทางที่ถูกขวาง


## โครงสร้างเอาต์พุต { #output-layout }

ผลลัพธ์ถูกจัดระเบียบตามคอมพิวเตอร์และหมวดหมู่การตรวจสอบ ไดเรกทอรีหมวดหมู่ใช้
PascalCase เช่น `Accounts` และ `FileAndFolderOpening` ตารางของ Amcache และ Jump Lists จะอยู่
ในหมวดหมู่ของตนโดยตรงพร้อมคำนำหน้าชื่อไฟล์ ชื่อไดเรกทอรีของคอมพิวเตอร์จะคงการสะกดตามที่ค้นพบ ไดเรกทอรีเอาต์พุต
ที่มีอยู่แล้วจะไม่ถูกย้ายโครงสร้าง ให้เลือกไดเรกทอรีเอาต์พุตใหม่เมื่อเปลี่ยนจากรูปแบบการตั้งชื่อแบบเก่า พาร์เซอร์แต่ละตัวจะเขียน
ไฟล์ `<artifact>-logs.txt` ไว้ข้างผลลัพธ์ รวมถึงกรณีที่อาร์ติแฟกต์ไม่ได้ถูกเก็บรวบรวม
CSV แยกตารางออกจากกันเพราะแต่ละอาร์ติแฟกต์มีคอลัมน์ต่างกัน:

```text
<OUTPUT>/
├── manifest.json
├── manifest.csv
├── HOST1/
│   ├── Accounts/
│   │   ├── Registry-LsaSecrets.csv
│   │   ├── Registry-LsaSecrets-logs.txt
│   │   ├── Registry-SAM.csv
│   │   └── Registry-SAM-logs.txt
│   ├── Applications/
│   │   ├── Registry-AppHistory.csv
│   │   ├── Registry-AppHistory-logs.txt
│   │   ├── Registry-InstalledPrograms.csv
│   │   └── Registry-InstalledPrograms-logs.txt
│   ├── BrowserActivity/
│   │   ├── Browser-Cookies.csv
│   │   ├── Browser-Downloads.csv
│   │   ├── Browser-FormHistory.csv
│   │   ├── Browser-History.csv
│   │   ├── Browser-logs.txt
│   │   ├── WebCache-Containers.csv
│   │   ├── WebCache-Cookies.csv
│   │   ├── WebCache-Downloads.csv
│   │   ├── WebCache-Entries.csv
│   │   ├── WebCache-History.csv
│   │   └── WebCache-logs.txt
│   ├── Communication/
│   │   ├── Outlook-Attachments.csv
│   │   ├── Outlook-Calendar.csv
│   │   ├── Outlook-Contacts.csv
│   │   ├── Outlook-Files.csv
│   │   ├── Outlook-Folders.csv
│   │   ├── Outlook-Items.csv
│   │   ├── Outlook-Recipients.csv
│   │   ├── Outlook-Tasks.csv
│   │   └── Outlook-logs.txt
│   ├── ExternalDevices/
│   │   ├── Registry-USB.csv
│   │   └── Registry-USB-logs.txt
│   ├── FileAndFolderOpening/
│   │   ├── JumpLists-AutomaticDestinations.csv
│   │   ├── JumpLists-CustomDestinations.csv
│   │   ├── JumpLists-logs.txt
│   │   ├── LNK.csv
│   │   ├── LNK-logs.txt
│   │   ├── Registry-Office.csv
│   │   ├── Registry-Office-logs.txt
│   │   ├── Registry-MRU.csv
│   │   ├── Registry-MRU-logs.txt
│   │   ├── Registry-OpenSaveMRU.csv
│   │   ├── Registry-OpenSaveMRU-logs.txt
│   │   ├── Registry-RecentDocs.csv
│   │   ├── Registry-RecentDocs-logs.txt
│   │   ├── Registry-ShellBags.csv
│   │   └── Registry-ShellBags-logs.txt
│   ├── FileExistence/
│   │   ├── RecycleBin.csv
│   │   ├── RecycleBin-logs.txt
│   │   ├── Registry-Amcache-Devices.csv
│   │   ├── Registry-Amcache-Files.csv
│   │   ├── Registry-Amcache-Findings.csv
│   │   ├── Registry-Amcache-Summary.csv
│   │   ├── Registry-Amcache-logs.txt
│   │   ├── Registry-ShimCache.csv
│   │   └── Registry-ShimCache-logs.txt
│   ├── FileSystem/
│   │   ├── Boot.csv
│   │   ├── Boot-logs.txt
│   │   ├── I30.csv
│   │   ├── I30-logs.txt
│   │   ├── LogFile.csv
│   │   ├── LogFile-logs.txt
│   │   ├── MFT.csv
│   │   ├── MFT-logs.txt
│   │   ├── SecureSDS.csv
│   │   ├── SecureSDS-logs.txt
│   │   ├── UsnJournal.csv
│   │   └── UsnJournal-logs.txt
│   ├── NetworkActivity/
│   │   ├── Registry-Networks.csv
│   │   ├── Registry-Networks-logs.txt
│   │   ├── SRUM-NetworkConnections.csv
│   │   ├── SRUM-NetworkUsages.csv
│   │   └── SRUM-PushNotifications.csv
│   ├── LateralMovement/
│   │   ├── Registry-RemoteAccess.csv
│   │   ├── Registry-RemoteAccess-logs.txt
│   │   ├── UAL-Clients.csv
│   │   ├── UAL-RoleAccess.csv
│   │   ├── UAL-SystemIdentity.csv
│   │   └── UAL-logs.txt
│   ├── Persistence/
│   │   ├── Registry-BCD.csv
│   │   ├── Registry-BCD-logs.txt
│   │   ├── Registry-Persistence.csv
│   │   ├── Registry-Persistence-logs.txt
│   │   ├── Registry-Services.csv
│   │   ├── Registry-Services-logs.txt
│   │   ├── Registry-TaskCache.csv
│   │   └── Registry-TaskCache-logs.txt
│   ├── ProcessExecution/
│   │   ├── Registry-AppCompat.csv
│   │   ├── Registry-AppCompat-logs.txt
│   │   ├── Registry-FeatureUsage.csv
│   │   ├── Registry-FeatureUsage-logs.txt
│   │   ├── Registry-BAM.csv
│   │   ├── Registry-BAM-logs.txt
│   │   ├── Registry-MuiCache.csv
│   │   ├── Registry-MuiCache-logs.txt
│   │   ├── Prefetch.csv
│   │   ├── Prefetch-logs.txt
│   │   ├── PrefetchTimeline.csv
│   │   ├── PrefetchTimeline-logs.txt
│   │   ├── SRUM-AppResourceUseInfo.csv
│   │   ├── SRUM-AppTimelineProvider.csv
│   │   ├── Registry-UserAssist.csv
│   │   └── Registry-UserAssist-logs.txt
│   ├── Registry/
│   │   ├── Registry-Dump.csv
│   │   └── Registry-Dump-logs.txt
│   ├── SecurityProducts/
│   │   ├── Registry-SecuritySettings.csv
│   │   ├── Registry-SecuritySettings-logs.txt
│   │   ├── DefenderQuarantine.csv
│   │   ├── DefenderQuarantine-logs.txt
│   │   └── DefenderQuarantined/  # only with -x
│   └── SystemInformation/
│       ├── Registry-AuditPolicy.csv
│       ├── Registry-AuditPolicy-logs.txt
│       ├── Registry-SystemInfo.csv
│       ├── Registry-SystemInfo-logs.txt
│       ├── SRUM-IdentityMap.csv
│       ├── SRUM-VfuProvider.csv
│       └── SRUM-logs.txt
└── HOST2/
    └── …
```


| หมวดหมู่ | อาร์ติแฟกต์ |
|---|---|
| `Accounts` | บัญชีภายในเครื่อง SAM, ชื่อและเวลาของ LSA secrets |
| `Applications` | ประวัติแอปพลิเคชัน (ร่องรอยของเครื่องมือจากบุคคลที่สาม) |
| `Applications` | โปรแกรมที่ติดตั้ง |
| `BrowserActivity` | ประวัติ การดาวน์โหลด คุกกี้ และประวัติฟอร์มของเว็บเบราว์เซอร์ พร้อมรายการที่ถูกลบซึ่งกู้คืนได้ รวมถึง WebCache ของ Internet Explorer และ Edge รุ่นเก่า |
| `Communication` | ไฟล์ PST, OST, MSG และ OFT ของ Outlook: โฟลเดอร์ รายการ ผู้รับ สิ่งที่แนบ ผู้ติดต่อ รายการปฏิทิน และงาน |
| `ExternalDevices` | อุปกรณ์และวอลุ่ม USB |
| `FileAndFolderOpening` | Jump Lists, ทางลัด LNK, Microsoft Office, รายการ MRU, กล่องโต้ตอบ Open และ Save As, RecentDocs, ShellBags |
| `FileExistence` | Amcache, ถังรีไซเคิล, ShimCache |
| `FileSystem` | `$Boot`, `$I30`, `$LogFile`, `$MFT`, `$Secure:$SDS`, USN Journal |
| `NetworkActivity` | โปรไฟล์เครือข่าย อินเทอร์เฟซ แชร์ port proxy และเส้นทาง; การเชื่อมต่อ ปริมาณการรับส่งข้อมูล และ push notification ของ SRUM |
| `LateralMovement` | การเชื่อมต่อขาออกของผู้ใช้ผ่าน Remote Desktop, PuTTY และ WinSCP, ไดรฟ์ที่แมปและแชร์ รวมถึงการบันทึกการเข้าถึงของผู้ใช้ (ไคลเอนต์ของบทบาทต่าง ๆ บนเซิร์ฟเวอร์) |
| `Persistence` | การกำหนดค่าบูต (BCD), การเริ่มอัตโนมัติผ่านรีจิสทรี, บริการ, TaskCache |
| `ProcessExecution` | ความเข้ากันได้ของแอปพลิเคชัน, BAM และ DAM, FeatureUsage และรายการที่ปักหมุด, MuiCache, Prefetch, ไทม์ไลน์ Prefetch, กิจกรรมแอปพลิเคชัน/CPU ของ SRUM, UserAssist |
| `Registry` | คีย์และค่าของไฮฟ์ทั่วไป รวมถึงเรคคอร์ดที่กู้คืน |
| `SecurityProducts` | เมทาดาทาการกักกันของ Defender และเนื้อหาที่แยกออกมา (ไม่บังคับ); การตั้งค่าความปลอดภัย (Defender, UAC, การป้องกันข้อมูลรับรอง, RDP, ไฟร์วอลล์, การบันทึก) |
| `SystemInformation` | ชื่อ เวอร์ชัน Windows เขตเวลา การปิดเครื่องครั้งล่าสุด โปรไฟล์ SID และนโยบายการตรวจสอบ; พลังงาน/ความจุ, identity map และ provider ที่ไม่รู้จักของ SRUM |

หมวดหมู่ช่วยจัดระเบียบการตรวจสอบ แต่ไม่ใช่หลักฐานของกิจกรรมใดกิจกรรมหนึ่ง ตัวอย่างเช่น
รายการ Amcache และ ShimCache เพียงลำพังไม่ได้ยืนยันการเรียกใช้งาน SRUM บันทึกการใช้ทรัพยากร
ให้ศึกษาคู่มืออาร์ติแฟกต์ก่อนสรุปว่าโปรแกรมได้ทำงาน

CSV, JSON และ JSONL ใช้หมวดหมู่เดียวกัน ไฟล์ `<artifact>-logs.txt` หนึ่งไฟล์ต่องานพาร์เซอร์หนึ่งงานจะมี
คำสั่งที่ใช้จริงและข้อความต่าง ๆ เช่น `HOST1/FileSystem/MFT-logs.txt` และ
`HOST1/ProcessExecution/Prefetch-logs.txt` JSON และ JSONL ใช้ชื่อไฟล์บันทึกเดียวกัน
Amcache และ Jump Lists แต่ละตัวมีบันทึกที่ใช้ร่วมกันหนึ่งไฟล์อยู่ข้างตารางที่มีคำนำหน้า ตาราง provider ของ SRUM จะถูกวางโดยตรง
ใน `NetworkActivity`, `ProcessExecution` และ `SystemInformation` พร้อมไฟล์
`SystemInformation/SRUM-logs.txt` ที่ใช้ร่วมกัน ไม่มีการสร้างไดเรกทอรีย่อย `logs/`

ชื่อไฟล์ CSV เริ่มต้นได้แก่ `Registry-SAM.csv`, `LNK.csv`, `RecycleBin.csv`, `Registry-RecentDocs.csv`, `Registry-ShimCache.csv`, `LogFile.csv`,
`I30.csv`, `MFT.csv`, `UsnJournal.csv`, `Boot.csv`, `SecureSDS.csv`, `Registry-Networks.csv`, `Registry-Persistence.csv`, `Registry-Services.csv`, `Registry-TaskCache.csv`, `Registry-BCD.csv`,
`Registry-SecuritySettings.csv`,
`PrefetchTimeline.csv`, `Prefetch.csv`, `Registry-UserAssist.csv`, `Registry-AppCompat.csv`, `Registry-Office.csv`, `Registry-FeatureUsage.csv`, `Registry-AppHistory.csv`, `Registry-MRU.csv`, `Registry-SystemInfo.csv`, `Registry-RemoteAccess.csv`, `Registry-OpenSaveMRU.csv`, `Registry-InstalledPrograms.csv`, `Registry-BAM.csv`, `Registry-MuiCache.csv`, `Registry-ShellBags.csv`, `Registry-USB.csv`, `Registry-SecuritySettings.csv`, `Registry-AuditPolicy.csv`, `Registry-Dump.csv` และ
`DefenderQuarantine.csv` ตารางที่ได้จากรีจิสทรีใช้คำนำหน้า `Registry-` รวมถึง
`Registry-ShimCache.csv`, `Registry-UserAssist.csv`, `Registry-BAM.csv` และ `Registry-SystemInfo.csv`
คีย์และค่าของไฮฟ์ทั่วไปอยู่ใน `Registry-Dump.csv` ตาราง Amcache ได้แก่
`FileExistence/Registry-Amcache-Devices.csv`, `Registry-Amcache-Files.csv`,
`Registry-Amcache-Findings.csv` และ `Registry-Amcache-Summary.csv` Jump Lists ใช้
`FileAndFolderOpening/JumpLists-AutomaticDestinations.csv` และ
`JumpLists-CustomDestinations.csv` ไม่มีไดเรกทอรีย่อยสำหรับ Amcache หรือ Jump Lists
ตาราง SRUM ใช้ชื่อไฟล์ `SRUM-` ไว้ใต้หมวดหมู่ของตนโดยตรง เช่น
`NetworkActivity/SRUM-NetworkConnections.csv`
ตารางของการบันทึกการเข้าถึงของผู้ใช้อยู่ใน `LateralMovement` โดยแยกหนึ่งไฟล์ต่อหนึ่งตาราง เช่น
`UAL-Clients.csv` และ `UAL-SystemIdentity.csv` พร้อม `UAL-logs.txt` ดู
[การบันทึกการเข้าถึงของผู้ใช้ (UAL)](../artifacts/ual.md#automatic-parsing)
ตารางของเว็บเบราว์เซอร์อยู่ใน `BrowserActivity` ได้แก่ `Browser-History.csv`, `Browser-Downloads.csv`,
`Browser-Cookies.csv` และ `Browser-FormHistory.csv` พร้อม `Browser-logs.txt` ดู
[เว็บเบราว์เซอร์](../artifacts/browser.md#automatic-parsing)
ตาราง WebCache ของ Internet Explorer และ Edge รุ่นเก่าก็อยู่ที่นี่เช่นกัน ได้แก่ `WebCache-History.csv`,
`WebCache-Downloads.csv`, `WebCache-Cookies.csv`, `WebCache-Entries.csv` และ `WebCache-Containers.csv`
พร้อม `WebCache-logs.txt` ดู [Internet Explorer และ Edge รุ่นเก่า](../artifacts/webcache.md#automatic-parsing)
ตารางของ Outlook อยู่ใน `Communication` ได้แก่ `Outlook-Files.csv`, `Outlook-Folders.csv`,
`Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`, `Outlook-Contacts.csv`,
`Outlook-Calendar.csv` และ `Outlook-Tasks.csv` พร้อม `Outlook-logs.txt` ดู
[Outlook](../artifacts/outlook.md#automatic-parsing)
ดู[ตารางการจัดวาง SRUM ทั้งหมด](../artifacts/srum.md#categorized-auto-parse-output)

บันทึกใช้ชื่อตามชื่ออาร์ติแฟกต์ที่ขึ้นต้นด้วยตัวพิมพ์ใหญ่ เช่น `Registry-SAM-logs.txt`, `PrefetchTimeline-logs.txt`
และ `Registry-UserAssist-logs.txt` ไฟล์ JSON/JSONL ใช้ชื่อหลักเดียวกัน เช่น `MFT.jsonl` ส่วน
ตัวระบุ `Artifact` ของเรคคอร์ดสำหรับการสอบถามคือ `mft` เอาต์พุตแบบหลายตารางจะมี
`Registry-Amcache-logs.txt` หรือ `JumpLists-logs.txt` อยู่ข้างตาราง บันทึก SRUM ที่ใช้ร่วมกัน
อยู่ใน `SystemInformation`


`manifest.json` บันทึก `Hostname` ดั้งเดิมที่ค้นพบ ตัวตน `Comp` ของเอาต์พุต/เรคคอร์ดที่ไม่ซ้ำกัน
ที่มาของชื่อโฮสต์ พาธของหลักฐาน `Category` ของอาร์ติแฟกต์ บรรทัดคำสั่ง ระยะเวลา exit
code พาธของผลลัพธ์ พาธของบันทึก ไทม์ไลน์ JSONL ที่ร้องขอและข้อผิดพลาดในการส่งออก (ถ้ามี) มีการร้องขอให้แยกไฟล์ของ Defender หรือไม่
และแต่ละงาน
สำเร็จ ถูกข้าม หรือล้มเหลว

Auto-parse ไม่เขียนพาธแบบสัมบูรณ์ใด ๆ ดังนั้นจึงสามารถแชร์ผลลัพธ์ได้โดยไม่เปิดเผยว่าหลักฐาน
ผลลัพธ์ หรือ Juntei อยู่ที่ใดบนระบบที่ใช้วิเคราะห์

- พาธของหลักฐานเป็นพาธสัมพัทธ์จากไดเรกทอรีหลักฐาน เทอร์มินัลแสดงคอมพิวเตอร์แต่ละเครื่องพร้อม
  ไดเรกทอรีในหลักฐาน (`HOST1: ./HOST1_F`) และพาร์เซอร์ทำงานในไดเรกทอรีหลักฐานและ
  ได้รับพาธในรูปแบบดังกล่าว คอลัมน์พาธต้นทางของผลลัพธ์ (`SourceFile`, `SourcePath`, …) บันทึก
  และ `EvidenceRoot` และ `WindowsDirectories` ของ manifest จะแสดงตำแหน่งของแต่ละไฟล์ใน
  หลักฐาน (`./HOST1_F/C/Windows/Prefetch/CMD.EXE-6D6290C5.pf`)
- พาธของผลลัพธ์ใน manifest และบันทึก (`Output`, `OutputFiles`, `Log`, `JSON-Timeline`, พาธ
  เอาต์พุตใน `Command`) และ `ExtractedFile` ของ Defender เป็นพาธสัมพัทธ์จากไดเรกทอรีเอาต์พุต
  (`./HOST1/ProcessExecution/Prefetch.csv`, `../siem.jsonl` สำหรับไทม์ไลน์ที่อยู่ข้างไดเรกทอรีนั้น) `Command`
  ขึ้นต้นด้วยชื่อโปรแกรม (`juntei`) พาธหลักฐานในคำสั่งเป็นพาธสัมพัทธ์จากไดเรกทอรีหลักฐาน
  และพาธเอาต์พุตเป็นพาธสัมพัทธ์จากไดเรกทอรีเอาต์พุต
- `EvidenceDirectory` และ `OutputDirectory` คือชื่อของไดเรกทอรีทั้งสอง `ManifestVersion` 2
  ระบุ manifest ที่บันทึกพาธผลลัพธ์จากไดเรกทอรีเอาต์พุต manifest รุ่นก่อนหน้าไม่มี
  `ManifestVersion` และบันทึกพาธตามที่ระบุให้กับ `-o` การเรียกใช้ซ้ำด้วย `-C` อ่านได้ทั้งสองแบบ
- เทอร์มินัลแสดงพาธเอาต์พุตหรือไทม์ไลน์ตามที่ระบุเมื่อเป็นพาธสัมพัทธ์ และแสดงจาก
  ไดเรกทอรีปัจจุบันเมื่อเป็นพาธสัมบูรณ์ (`../results/`)

หลังจากจำนวนโพรเซสแบบขนาน เทอร์มินัลจะแสดงขนาดรวมของไฟล์ของ
ระบบ Windows ที่ค้นพบ (`Total filesize: 26.1 MiB`)

ข้อความแสดงความคืบหน้าจะแสดงคอมพิวเตอร์และอาร์ติแฟกต์ที่กำลังทำงานอยู่ ตามด้วยขนาดอินพุต
โดยประมาณ เช่น `HOST1 / prefetch (~2.0 MiB)` กลุ่มของไฟล์จะใช้ขนาดรวมกัน ส่วนงาน
MFT, LogFile หรือ USN Journal แต่ละงานจะแสดงขนาดอินพุตของตนเอง ค่าประมาณไม่รวมบันทึกธุรกรรม
เสริมและไฟล์สำหรับเสริมข้อมูล และไม่ได้คาดการณ์ขนาดของเอาต์พุตที่พาร์สแล้ว

`manifest.csv` ให้ดัชนีงานเดียวกันในรูปตารางที่ใช้กับสเปรดชีตได้สะดวก โดยมีหนึ่งแถวต่อ
งานพาร์เซอร์หนึ่งงาน ประกอบด้วย `Comp`, `Hostname` ดั้งเดิม, `FQDN`, `HostnameSource`, `EvidenceRoot`,
`Artifact`, `Category`, `OutputType`, `Purpose`, `Status`, `DurationMs`, `ExitCode`, `Output`, `OutputFiles`, `Log`,
`Error` และ `Command` `Purpose` เป็น `artifact` สำหรับผลลัพธ์ที่จัดหมวดหมู่ หรือ `json_timeline` สำหรับ
รอบการทำงาน JSONL แบบเนทีฟเพิ่มเติมที่ใช้ร่วมกับ CSV `OutputType` คือรูปแบบเอาต์พุตจริงของพาร์เซอร์
`Command` เป็นอาร์เรย์ JSON ของไฟล์ปฏิบัติการและอาร์กิวเมนต์ ซึ่งเก็บรักษาช่องว่างและเครื่องหมายคำพูดไว้
สำหรับ SRUM `Output` คือไดเรกทอรีของคอมพิวเตอร์ `OutputFiles` เป็นอาร์เรย์ JSON ของพาธเอาต์พุต
ของ provider ที่แน่นอน และ `Category` ระบุตำแหน่งบันทึกที่ใช้ร่วมกัน (`SystemInformation`)
ไฟล์ถูกจัดหมวดหมู่ทีละไฟล์ `OutputFiles` ยังปรากฏใน manifest แบบ JSON ด้วย

กรอง `Status` เป็น `skipped` หรือ `failed` เพื่อค้นหางานที่ต้องตรวจทาน `Error` ของงานที่ถูกข้าม
อาจว่างเปล่า โดยมีคำอธิบายอยู่ในไฟล์ `Log` ของงานนั้น

ข้อมูลสรุปในเทอร์มินัลมีจำนวนและขนาดรวมของไฟล์ปกติที่อยู่ภายใต้
ไดเรกทอรีเอาต์พุตในขณะนั้น รวมถึง manifest ทั้งสอง บันทึกของพาร์เซอร์ และไฟล์ที่แยกออกมา จะไม่ติดตาม
symbolic link ขนาดจะปรับหน่วยโดยอัตโนมัติด้วยตัวจัดรูปแบบเดียวกับ `Total filesize`
โดยใช้หน่วยเช่น `KiB`, `MiB` และ `GiB`:

```text
Results saved: output/ ( 125 files / 40.5 MiB )
```

ตาราง Amcache และ SRUM ใช้ `-Source1`, `-Source2` ฯลฯ เมื่อมีการเก็บรวบรวมฐานข้อมูลต้นทาง
หลายชุด บันทึกของพาร์เซอร์ที่ใช้ร่วมกันจะบันทึกการจับคู่พาธดั้งเดิมกับพาธสุดท้าย และเรคคอร์ด JSON/JSONL
จะเก็บแหล่งที่มาของต้นทางไว้ งาน Amcache/Jump Lists แบบ CSV จะแสดงพาธตารางที่แน่นอนใน
`OutputFiles` ของ manifest ส่วน `Output` คือไดเรกทอรีหมวดหมู่การตรวจสอบ การส่งออก JSON และ JSONL
ของ Amcache และ Jump Lists เป็นไฟล์เดียวอยู่แล้ว โดยมีชื่อว่า `Registry-Amcache.json[l]` และ
`JumpLists.json[l]` ไทม์ไลน์ `-j` แบบรวมจะเก็บเนื้อหาเรคคอร์ดและค่า `Artifact` เหมือนเดิม
`--clobber` ใช้ manifest ก่อนหน้าเพื่อล้างเอาต์พุตที่บันทึกไว้ รวมถึงชื่อแบบเก่า
ขณะที่ยังคงไฟล์ในหมวดหมู่ที่ไม่เกี่ยวข้องไว้ ชุดข้อมูลวอลุ่มเดียวตามปกติจะเขียนเรคคอร์ด `$MFT` และ USN Journal ลงใน
`FileSystem/MFT.csv` และ `FileSystem/UsnJournal.csv` โดยตรง วอลุ่มเพิ่มเติมที่เก็บรวบรวมมาจะใช้ไฟล์
ข้างเคียงที่มีหมายเลข เช่น `FileSystem/MFT-2.csv` และ `FileSystem/MFT-2-logs.txt`

## การนำเข้า JSON และ SIEM { #json-and-siem-ingestion }

เรคคอร์ด JSON หรือ JSONL ทุกรายการจะได้รับฟิลด์ระดับบนสุดสองฟิลด์ ได้แก่

- `Comp`: ระบบต้นทาง
- `Artifact`: พาร์เซอร์ของ Juntei ที่สร้างเรคคอร์ด เช่น `mft` หรือ `prefetch-timeline`

`-j, --json-timeline <FILE>` สร้างไฟล์ JSONL เพิ่มเติมหนึ่งไฟล์ ไม่ว่ารูปแบบผลลัพธ์แต่ละรายการ
จะเป็นแบบใด เอาต์พุตที่จัดหมวดหมู่และบันทึกทั้งหมดจะยังคงอยู่:

```bash
# Default CSV tables plus a SIEM timeline
juntei auto-parse -d evidence -o parsed -j parsed/timeline.jsonl

# Pretty JSON results plus a SIEM timeline
juntei auto-parse -d evidence -o parsed-json -t json -j parsed-json/timeline.jsonl

# Individual JSONL results plus a consolidated JSONL timeline
juntei auto-parse -d evidence -o parsed-jsonl -t jsonl -j parsed-jsonl/timeline.jsonl
```

ชื่อไฟล์จะถูกใช้ตามที่ระบุ พาธสัมพัทธ์จะอ้างอิงจากไดเรกทอรีทำงานปัจจุบัน
ไฟล์อาจอยู่ที่รากของเอาต์พุต ไดเรกทอรีย่อยของเอาต์พุตอื่นที่ไม่ได้สงวนไว้ หรืออยู่นอกไดเรกทอรี
เอาต์พุตก็ได้ หลักฐาน manifest ผลลัพธ์/บันทึกของแต่ละคอมพิวเตอร์ และไดเรกทอรีชั่วคราวของพาร์เซอร์ไม่สามารถ
ใช้เป็นปลายทางได้ ไฟล์ที่มีอยู่แล้วต้องใช้ `-C` และการส่งออกที่ล้มเหลวจะไม่แตะต้องไฟล์นั้น

ใน **โหมด CSV** Juntei จะทำรอบ JSONL แบบเนทีฟเพิ่มเติม เพื่อให้ไฟล์ SIEM เก็บรักษาชนิด
ตัวเลขและบูลีน ข้อมูลแบบซ้อน ฟิลด์ดิบ และเมทาดาทาที่ตาราง CSV อาจละเว้นไว้
งานเพิ่มเติมเหล่านี้ใช้ขีดจำกัดการทำงานพร้อมกัน `-T` เดียวกัน ไม่แยกเนื้อหาที่ถูกกักกันซ้ำ และ
มีบันทึกประกอบ `*-json-timeline-logs.txt` แยกต่างหาก manifest ทั้งสองแยกแยะวัตถุประสงค์และ
รูปแบบเอาต์พุตของงานเหล่านี้ ไฟล์ JSONL ชั่วคราวจะถูกลบหลังการทำงานสำเร็จ และจะถูกเก็บไว้หาก
พาร์เซอร์หรือการส่งออกล้มเหลว ให้วางแผนเผื่อเวลาประมวลผลและพื้นที่ดิสก์ชั่วคราวที่เพิ่มขึ้น

ใน **โหมด JSON** Juntei จะสตรีมผลลัพธ์ที่มีอยู่ให้เป็น JSONL แบบกะทัดรัดโดยไม่พาร์ส
หลักฐานซ้ำ รายงาน Amcache จะถูกขยายเป็นเรคคอร์ดสรุป ข้อค้นพบ ไฟล์ และอุปกรณ์แยกรายการ
แบบเดียวกับ JSONL แบบเนทีฟ ใน **โหมด JSONL** เรคคอร์ดที่มีคำอธิบายประกอบอยู่แล้วจะถูกรวมเข้าด้วยกันโดยตรง
ทุกโหมดใช้โครงสร้างเรคคอร์ด SIEM เดียวกัน ชุดข้อมูลเก็บ timestamp และฟิลด์ของอาร์ติแฟกต์
แบบเนทีฟไว้ โดยไม่สร้าง timestamp เหตุการณ์ร่วมขึ้นมาเอง หรือเรียงลำดับอาร์ติแฟกต์ต่างชนิดกันแบบรวมทั้งหมด

```bash
jq -c 'select(.Comp == "HOST1" and .Artifact == "prefetch")' parsed/timeline.jsonl
jq -r 'select(.Artifact == "mft") | .FullPath? // empty' parsed/timeline.jsonl
```

`manifest.json` บันทึกพาธ `JSON-Timeline` ที่ร้องขอและ `JSON-TimelineError` (ถ้ามี) เรคคอร์ดงาน
ใน manifest นี้และ `manifest.csv` ยังคงชี้ไปยังเอาต์พุตของอาร์ติแฟกต์ที่เก็บไว้ งานที่ทำเฉพาะไทม์ไลน์ CSV
ที่สำเร็จจะอ้างอิงไฟล์ JSONL สุดท้ายแทนไฟล์ชั่วคราวที่ถูกลบไปแล้ว หากพาร์เซอร์ใดล้มเหลว
การทำงานจะจบลงโดยไม่สำเร็จ และไทม์ไลน์จะมีเฉพาะผลลัพธ์ที่สำเร็จ ให้ตรวจสอบ manifest
และบันทึกของหมวดหมู่ก่อนถือว่าไทม์ไลน์สมบูรณ์ ข้อมูลสรุปจำนวน/ขนาดไฟล์ครอบคลุมไฟล์ภายใน
ไดเรกทอรีเอาต์พุต ไทม์ไลน์ที่อยู่ภายนอกจะถูกรายงานแยกเป็น `JSONL timeline: <FILE>`

## ประสิทธิภาพ { #performance }

งานพาร์เซอร์แต่ละงานเป็นโพรเซสแยกกัน ดังนั้นอาร์ติแฟกต์ที่ใช้ CPU มากจึงทำงานพร้อมกันได้ และความเสียหายใน
พาร์เซอร์หนึ่งตัวจะไม่ทำให้การทำงานทั้งหมดหยุดลง โดยค่าเริ่มต้น Juntei ใช้จำนวน CPU เชิงตรรกะ ให้จำกัด
การทำงานพร้อมกันเมื่อหลักฐานอยู่บนดิสก์ที่ช้าหรือระบบมีหน่วยความจำน้อย:

```bash
juntei auto-parse -d evidence -o parsed -T 4
```

## การใช้งาน { #usage }

```text
Version: 1.1.0 (CODE BLUE Release)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei auto-parse -d <EVIDENCE> -o <OUTPUT> [OPTIONS]

Input:
  -d, --directory <EVIDENCE>  Evidence directory containing one or more collected Windows systems
      --max-depth <DEPTH>     Maximum directory depth at which to discover a Windows directory [default: 10]
      --vhdx                  Parse VHDX images natively on Linux, macOS and Windows (read-only; temporary disk space required)
      --e01                   Parse E01/Ex01 images natively (split images from their first segment; temporary disk space required)
      --temp-dir <DIR>        Directory for private image staging (default: output filesystem)

Output:
  -o, --output <DIR>                  New directory in which to create per-computer results
  -t, --output-type <TYPE>            Output type: csv (default), json, or jsonl [default: csv]
  -j, --json-timeline <FILE>          Create a single JSONL timeline to import into a SIEM
  -Z, --timezone <OFFSET>             Display parsed timestamps using a fixed UTC offset (UTC by default) [default: +00:00]
  -C, --clobber                       Overwrite outputs in an existing directory (never deletes unrelated files)
  -x, --extract-defender-quarantined  Extract Defender-quarantined files

Performance:
  -T, --threads <N>  Parser processes to run at once (default: logical CPU count)

Parsing:
      --code-page <NUMBER>  Windows code page for ANSI strings in LNK files, Jump Lists, ShellBags and OpenSaveMRU [default: 1252]
      --mft-slack           Recover the file names left in the slack of $MFT records
      --no-reconstruct-filesystem  Skip the browsable MFT/I30 filesystem reconstruction

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

ควรใช้ไดเรกทอรีเอาต์พุตใหม่เมื่อทำได้ `-C` จะลบหรือเขียนทับเฉพาะเป้าหมายที่สร้างขึ้น
สำหรับการทำงานครั้งนี้เท่านั้น ดังนั้นอาร์ติแฟกต์ที่ไม่มีอยู่แล้วจะไม่ทิ้งผลลัพธ์ที่ล้าสมัยไว้ คำสั่งนี้ไม่ลบ
ไฟล์ที่ไม่เกี่ยวข้องหรือโครงสร้างเอาต์พุตทั้งหมด นอกจากนี้ยังลบไดเรกทอรี
`SecurityProducts/DefenderQuarantined/` ของแต่ละคอมพิวเตอร์ และไดเรกทอรี `defender-quarantined/` ที่สงวนไว้
จากโครงสร้างแบบแบนเดิม ก่อนการพาร์ส ไม่ว่าการทำงานครั้งใหม่จะใช้ `-x` หรือไม่

ให้ใช้ไดเรกทอรีเอาต์พุตใหม่เมื่อเปลี่ยนจากโครงสร้างแบบแบนแบบเก่า ไฟล์ CSV/JSON เดิม
จะไม่ถูกย้ายหรือลบโดยอัตโนมัติ

!!! danger "ให้ถือว่าเนื้อหาของ Defender ที่แยกออกมาเป็นมัลแวร์"
    `-x, --extract-defender-quarantined` เขียนเนื้อหาที่ถูกกักกันลงดิสก์ภายใต้ไดเรกทอรี
    `SecurityProducts/DefenderQuarantined/` ของแต่ละคอมพิวเตอร์ สตรีมไบนารีที่กู้คืนจะลงท้ายด้วย `.quarantined` และ Juntei
    จะลบสิทธิ์การเรียกใช้งานออก ตัวบอกความปลอดภัยแบบไบนารีแต่ละรายการยังได้รับรายงาน `.security_descriptor.txt` ที่อ่านได้
    ซึ่งมีความเป็นเจ้าของ สิทธิ์ และ SDDL ห้ามเรียกใช้ payload ที่กู้คืนมาโดยเด็ดขาด ให้ใช้ระบบวิเคราะห์
    ที่แยกออกจากเครือข่าย และคาดว่าซอฟต์แวร์แอนติไวรัสหรือ EDR จะแจ้งเตือนหรือลบเนื้อหาที่กู้คืนมา

ดู[การอ่านตัวบอกความปลอดภัยที่กู้คืน](../artifacts/defender-quarantine.md#reading-the-recovered-security-descriptor)
สำหรับฟิลด์ของรายงานและข้อจำกัดทางนิติวิทยาศาสตร์

Auto-parse ยังอ่านการเชื่อมต่อระยะไกลของผู้ใช้ (RDP, PuTTY, WinSCP และไดรฟ์ที่แมป) ด้วย

## สร้างซุปเปอร์ไทม์ไลน์ตามลำดับเวลา { #build-a-chronological-super-timeline }

หลังการพาร์ส ให้ใช้ [`super-timeline`](super-timeline.md) เพื่อขยาย timestamp ที่บันทึกไว้ให้เป็นเหตุการณ์ที่มีป้ายกำกับ
เพิ่ม `--sort` เพื่อเรียงลำดับเหตุการณ์เหล่านั้นรวมกันข้ามคอมพิวเตอร์และอาร์ติแฟกต์:

```bash
juntei super-timeline -d output/ -o SuperTimeline.csv --sort
```

การส่งออก `-j, --json-timeline` ที่มีอยู่เดิมยังคงเป็นไฟล์ JSONL ที่รวมเรคคอร์ดเนทีฟไว้ด้วยกัน
ไฟล์นี้คง schema ของพาร์เซอร์แต่ละตัวไว้ โดยไม่ขยายฟิลด์ timestamp และไม่เรียงลำดับเหตุการณ์
ให้เก็บไฟล์นี้และผลลัพธ์ของพาร์เซอร์แต่ละตัวไว้สำหรับการตรวจทานในระดับหลักฐาน
