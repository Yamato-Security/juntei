# การตรวจสอบความถูกต้องของพาร์เซอร์ { #parser-validation }

crate พาร์เซอร์ที่ Juntei ดูแลได้รับการทดสอบกับหลักฐานสาธารณะและการพัฒนาที่เป็นอิสระ
การเปรียบเทียบเหล่านี้ช่วยตรวจจับ regression และความแตกต่างในการตีความ แต่
ไม่ได้ทำให้พาร์เซอร์ใดไม่มีข้อผิดพลาด ให้เก็บรักษาหลักฐานต้นทาง ตรวจทานฟิลด์ปัญหาหรือ
ความเสียหายของแต่ละเรคคอร์ด และตรวจสอบข้อค้นพบที่มีผลสำคัญด้วยอาร์ติแฟกต์หรือการพัฒนาอื่น

ข้อความด้านล่างเก็บรักษาข้อมูลการตรวจสอบความถูกต้องที่เคยอยู่ใน README หลักของ Juntei
ดูรีโพซิทอรีพาร์เซอร์ที่ลิงก์ไว้สำหรับ test fixture และรายละเอียดการพัฒนาล่าสุด

## Amcache { #amcache }

Juntei ใช้ [ys-amcache-parser](https://github.com/Yamato-Security/ys-amcache-parser) ซึ่งเป็น
fork ภายใต้ Apache-2.0 ของ
[amcache-forensic](https://github.com/SecurityRonin/amcache-forensic) ของ Security Ronin
[เอกสารเอาต์พุต](https://github.com/Yamato-Security/ys-amcache-parser/blob/main/docs/output.md)
ของพาร์เซอร์นี้กำหนดทุกฟิลด์ของไฟล์ปฏิบัติการ อุปกรณ์ ข้อค้นพบ และข้อมูลสรุป

## ตารางไฟล์หลัก { #master-file-table }

crate mft ของ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) เป็น fork ที่ดูแลต่อจาก crate
[mft](https://github.com/omerbenamram/mft) ของ Omer Ben-Amram จากไฟล์ `$MFT` แปดไฟล์ ได้แก่ ตัวอย่างของ
crate, NIST hacking case และไฟล์จาก DFIR Artifact Museum หกไฟล์ แถวและคอลัมน์ CSV จำนวน 1,048,733 แถว
ตรงกับพาร์เซอร์อ้างอิงตามลำดับ การเปรียบเทียบครอบคลุมเอาต์พุตเริ่มต้น `--all-times`,
`--short-names` และ `--cutoff` โดยการแสดงผล timestamp เป็นความแตกต่างที่ตั้งใจไว้

## บันทึกธุรกรรม NTFS ($LogFile) { #ntfs-transaction-log-logfile }

crate logfile ของ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) ได้รับการดูแลต่อจาก
[Sootmark ntfslog](https://github.com/Sootmark/ntfslog) การตรวจสอบความถูกต้องครอบคลุมบันทึกที่สมบูรณ์หกชุดจาก DFIR
Artifact Museum, fixture สาธารณะของเวอร์ชัน 1.1 และ 2.0, เมทาดาทา redo/undo ที่ถอดรหัสแล้ว และการเขียน
journal ที่เปรียบเทียบกับข้อมูล NTFS ที่พาร์สโดยอิสระ ดูรีโพซิทอรีของพาร์เซอร์สำหรับรายละเอียด
ความครอบคลุมและความแตกต่างที่ทราบ

Juntei เพิ่มการทดสอบ CLI ที่เปรียบเทียบทุกฟิลด์ CSV กับ JSON และ JSONL บนวอลุ่มทดสอบสาธารณะของ go-ntfs
(2,129 เรคคอร์ด) ตรวจสอบข้อมูล redo/undo ที่แยกกันและความยาวของ payload ดิบ ทดสอบบันทึกที่เสียหาย
ปฏิเสธการเขียนทับหลักฐาน และป้องกันไม่ให้พาธ MFT ปัจจุบันไปตามหมายเลขเรคคอร์ดที่ถูกนำกลับมาใช้ใหม่
กรณี regression แยกแยะเรคคอร์ด MFT ที่ได้รับผลกระทบออกจากแอตทริบิวต์ที่เปิดอยู่ซึ่งใช้เขียนผ่าน
ตรวจสอบเมทาดาทาของ directory entry/data attribute แบบแบน และป้องกันการจับคู่ MFT ข้ามวอลุ่ม
การทดสอบ `auto-parse` แบบหลายวอลุ่มครอบคลุมไฟล์ CSV ที่แยกกันและไฟล์ JSONL ไฟล์เดียวพร้อมแหล่งที่มา

คำสั่งนี้ยังได้รับการตรวจสอบกับบันทึก Windows หกชุดที่เก็บรวบรวมเป็นการส่วนตัว: เรคคอร์ด CSV ที่ส่งออกทั้งหมด 708,465
รายการตรงกับ JSONL ทุกฟิลด์ และการส่งออก `auto-parse` ต่อโฮสต์ยังคงจำนวนเรคคอร์ดเท่าเดิม
หลักฐานต้นทางไม่ได้เผยแพร่ไปพร้อมกับรีโพซิทอรี การตรวจสอบเหล่านี้ยืนยันความสอดคล้องของเอาต์พุต
ไม่ได้ยืนยันว่าทุกธุรกรรมที่บันทึกไว้แสดงถึงการกระทำของผู้ใช้ที่เสร็จสมบูรณ์

## Prefetch { #prefetch }

[ys-prefetch-parser](https://github.com/Yamato-Security/ys-prefetch-parser) เป็น fork ที่ดูแลต่อจาก
[frnsc-prefetch](https://github.com/ForensicRS/frnsc-prefetch) ของ ForensicRS เอาต์พุตที่ถอดรหัสแล้วได้รับการ
เปรียบเทียบกับ [libscca](https://github.com/libyal/libscca) ซึ่งเป็นการพัฒนา Prefetch
โดยอิสระของ libyal

## USN Journal { #usn-journal }

crate usn ของ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) เป็น fork ที่ดูแลต่อจาก
[Sootmark usn](https://github.com/Sootmark/usn) บน journal จริงหกชุด พาธของไดเรกทอรีแม่แบบคำนึงถึงเวลาของ Juntei
ตรงกับพาร์เซอร์อิสระในทุกกรณีที่พาร์เซอร์นั้น resolve พาธได้ นอกจากนี้ Juntei
ยัง resolve เรคคอร์ดได้อีก 4,321 รายการที่การเปรียบเทียบไม่สามารถ resolve ได้ ตัว carver ไม่สร้าง
เรคคอร์ดเท็จใด ๆ เมื่อทดสอบกับข้อมูล `$MFT` ขนาด 880 MB

## ดัชนีไดเรกทอรี ($I30) { #directory-indexes-i30 }

โมดูล `i30` ของ crate mft ใน [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) อ่านเรคคอร์ด INDX
และ slack ของเรคคอร์ด บนสตรีม `$I30` สองชุดของวอลุ่มที่สร้างด้วย ntfs-3g (`test_files/i30/` ของ
Juntei) และบนตัวอย่าง `$I30` สาธารณะที่ Windows เขียนสามชุด รายการที่ใช้งานอยู่และรายการใน
slack ตรงกับของ [dfir_ntfs](https://github.com/msuhanov/dfir_ntfs) ของ Maxim Suhanov โดยมี
ชื่อและเวลาเหมือนกัน ในกรณีที่ dfir_ntfs รายงาน parent reference ที่เกิน 32 บิต (ไบต์ที่เขียนทับ
parent) Juntei จะปล่อย parent ว่างไว้

## บูตเซกเตอร์ ($Boot) { #boot-sector-boot }

crate boot ของ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) เป็นของ Yamato
Security เอง บนไฟล์ `$Boot` สาธารณะห้าไฟล์ของ DFIR Artifact Museum (Windows 10, 11 และ
Server 2012 R2) ทุกฟิลด์ตรงกับพาร์เซอร์อ้างอิง บนวอลุ่มที่สร้างด้วย `mkntfs` ที่มีเซกเตอร์ขนาด 512,
1,024 และ 4,096 ไบต์ และคลัสเตอร์ขนาด 4 KiB ถึง 64 KiB ขนาด คลัสเตอร์ของ `$MFT` และ `$MFTMirr`
และหมายเลขซีเรียลตรงกับ `fsstat` ของ The Sleuth Kit `$Boot` ของ Windows 11 จาก Museum
มีหมายเลขซีเรียลของวอลุ่มตรงกับที่ไฟล์ LNK ของโฮสต์เดียวกันบันทึกไว้ และไฟล์ทั้งห้ามี
boot code เหมือนกัน

## ตัวบอกความปลอดภัย ($Secure:$SDS) { #security-descriptors-securesds }

crate secure ของ [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) เป็นของ Yamato
Security เอง จากสตรีม `$SDS` ห้าชุดของ Museum (ตัวบอก 7,808 รายการ, ACE 60,781 รายการ)
security ID, แฮช, offset, เจ้าของ, กลุ่ม, control flag และจำนวน ACE ตรงกับพาร์เซอร์อ้างอิง
และชนิด แฟล็ก mask, SID และ object type ของทุก ACE ตรงกับ `dissect.ntfs` แฮชที่บันทึกไว้
ทุกค่าเป็นของตัวบอกนั้นจริง สำเนา mirror ทุกชุดตรงกับรายการของตน และตัวบอกทุกรายการมี
SDDL รวมถึงเงื่อนไขของ callback ACE ทั้ง 3,384 รายการ

## SRUM { #srum }

[ys-srum-parser](https://github.com/Yamato-Security/ys-srum-parser) พัฒนาต่อจาก crate
[ese](https://github.com/Sootmark/ese) และ [srum](https://github.com/Sootmark/srum) ของ Sootmark จาก
ฐานข้อมูลจริง 11 ชุด ทุกคอลัมน์ในรายการที่ส่งออก 45,255 รายการตรงกับการถอดรหัสโดยอิสระของ
ค่าที่ `dissect.esedb` เปิดเผย

## การบันทึกการเข้าถึงของผู้ใช้ { #user-access-logging }

`parse-ese ual` อ่านฐานข้อมูล UAL ด้วยตัวอ่าน ESE ตัวเดียวกัน จากฐานข้อมูลจริง 16 ชุด (308 แถว
โดย 145 แถวเป็นแถวไคลเอนต์) ทุกค่าตรงกับการถอดรหัสโดยอิสระของค่าที่ `dissect.esedb` เปิดเผย
ในทุกแถวไคลเอนต์ วันที่มีจำนวนคำขออยู่ในช่วงตั้งแต่วันตาม UTC ของ `InsertDate` ถึงวันตาม UTC ของ
`LastAccess` และผลรวมของจำนวนเหล่านั้นเท่ากับ `TotalAccesses`

## เว็บเบราว์เซอร์ { #web-browsers }

[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) เป็น fork ที่ดูแลต่อจาก crate
browser, sqlite และ plist ของ Sootmark ซึ่งได้รับเลือกหลังจากเปรียบเทียบกับไลบรารี Rust อีกตัวหนึ่ง
และกับ Plaso, Velociraptor (SQLiteHunter) และ Hindsight บนฐานข้อมูลชุดเดียวกัน `parse-sqlite browser`
ได้รับการเปรียบเทียบกับไลบรารี `sqlite3` ที่อ่านสำเนาของแต่ละฐานข้อมูลพร้อมไฟล์ `-wal` บนฐานข้อมูลสาธารณะ
38 ชุด ได้แก่ ไฟล์ทดสอบของ Plaso (Chrome 8 ถึง 68, Firefox 2 ถึง 118) โปรไฟล์ที่สร้างโดย Chromium 150
และ Firefox 140 และฐานข้อมูลทดสอบประวัติฟอร์มของ ys-browser-parser การเข้าชมทั้ง 293 รายการ
การดาวน์โหลด 21 รายการ คุกกี้ 692 รายการ และรายการฟอร์ม 41 รายการตรงกันทั้งหมด รายการที่ถูกลบ
ถูกกู้คืนเฉพาะในกรณีที่มีการลบรายการจริงเท่านั้น ได้แก่ หน้าเว็บสองหน้าและการเข้าชมสองรายการที่ถูกลบจาก
ประวัติของ Firefox 140 พร้อมเวลา URL และชื่อเรื่อง และรายการฟอร์มที่ถูกลบในฐานข้อมูลทดสอบ

## Internet Explorer และ Edge รุ่นเก่า (WebCache) { #internet-explorer-and-legacy-edge-webcache }

`parse-ese webcache` อ่าน `WebCacheV01.dat` ด้วย
[ys-browser-parser](https://github.com/Yamato-Security/ys-browser-parser) และได้รับการเปรียบเทียบกับตัวอ่าน ESE
ของ `dissect.database` บนฐานข้อมูล WebCache สาธารณะสองชุดของ Plaso ได้แก่ ชุดหนึ่งของ Internet Explorer 11
และอีกชุดหนึ่งของ Windows 10 ที่อยู่ในสถานะ dirty ซึ่งมีประวัติ การดาวน์โหลด และคุกกี้ของ Edge รุ่นเก่า
แถวทั้ง 4,156 แถวตรงกันทุกค่า ได้แก่ คอนเทนเนอร์ 44 รายการ รายการประวัติ 182 รายการ การดาวน์โหลด 6 รายการ
คุกกี้ 170 รายการ และรายการอื่น 3,754 รายการ การเปรียบเทียบนี้ถอดรหัสเรคคอร์ดการดาวน์โหลดด้วยวิธีของตนเอง
โดยอิสระ เรคคอร์ดที่ฐานข้อมูลทำเครื่องหมายว่าถูกลบไม่ได้ถูกนำมารวมทั้งสองฝั่ง

## Outlook { #outlook }

`parse-file outlook` อ่านไฟล์ PST, OST, `.msg` และ `.oft` ด้วย
[ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser) ซึ่งตัวอ่าน PST และ OST เป็น fork
ของ [outlook-pst-rs](https://github.com/microsoft/outlook-pst-rs) ของ Microsoft และตัวอ่าน `.msg`
เขียนขึ้นจากข้อกำหนดแบบเปิด (open specifications) ของ Microsoft

- **PST และ OST**: ไฟล์ PST และ OST สาธารณะ 66 ไฟล์ (Apache Tika, artemis, ตัวอย่างของ Aspose.Email,
  DFIR Artifact Museum, Enron, ตัวอย่างของ GroupDocs, java-libpst, ข้อมูลทดสอบของ libyal, outlook-pst-rs
  และ pst-extractor) ได้รับการเปรียบเทียบกับ libpff ทีละคุณสมบัติ ได้แก่ คุณสมบัติทุกตัวของทุกที่เก็บข้อมูล
  โฟลเดอร์ ข้อความ ผู้รับ และสิ่งที่แนบ และข้อมูลของสิ่งที่แนบทุกรายการ ข้อเท็จจริง 324,175 รายการตรงกัน
  และไม่มีรายการใดที่ต่างกันโดยไม่มีคำอธิบาย ส่วนที่เหลือคือสิ่งที่มีเพียง ys-outlook-parser อ่านได้
  (ข้อเท็จจริง 15,846 รายการของข้อความ ผู้รับ และสิ่งที่แนบที่ libpff อ่านไม่สำเร็จ และของที่เก็บข้อมูลหนึ่งที่
  libpff เปิดไม่ได้ และอีก 8,223 รายการของรายการที่ซ่อนอยู่) เซลล์ของผู้รับที่ libpff ให้ค่าว่างหรือศูนย์ทั้งที่แถว
  ระบุว่าไม่มีค่าเหล่านั้น (49,077) entry id และชนิดของโฟลเดอร์ที่คำนวณแบบเดียวกับ MAPI (2,004) และออบเจกต์
  กับข้อความที่ฝังอยู่ซึ่ง libpff ให้มาเป็นข้อมูลดิบ (480) มีสองไฟล์ที่ไม่มีตัวอ่านใดอ่านได้เลย
- **`.msg` และ `.oft`**: ไฟล์สาธารณะ 214 ไฟล์ ซึ่งมีข้อความ ผู้รับ และสิ่งที่แนบ 2,404 รายการ และคุณสมบัติ
  41,002 รายการ ได้รับการเปรียบเทียบกับ extract-msg ข้อเท็จจริง 110,114 รายการตรงกัน และทั้ง 791 รายการ
  ที่ต่างกันมีคำอธิบายทุกรายการ ได้แก่ 601 รายการเป็นข้อผิดพลาดของ extract-msg เกี่ยวกับเวลา 35 รายการมาจาก
  ไฟล์ ANSI ที่ extract-msg ถอดรหัสด้วยโค้ดเพจที่ผิด และที่เหลือคือสิ่งที่ extract-msg ละเว้นหรือเพิ่มเข้ามา
- **ข้อมูลนำเข้าที่เสียหาย**: ไฟล์ `.msg` ที่ถูกดัดแปลง (mutated) มากกว่า 260,000 ไฟล์ และ PST ที่ถูกดัดแปลง
  160,000 ไฟล์ รวมถึงไฟล์ที่สร้างขึ้นเป็นพิเศษ (crafted) ถูกอ่านได้โดยไม่เกิดการแครช การค้าง หรือการใช้
  หน่วยความจำอย่างไม่จำกัด

## ทางลัด LNK { #lnk-shortcuts }

[ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser) เป็น fork ที่ดูแลต่อจาก crate
[shell](https://github.com/Sootmark/shell) ของ Sootmark ได้รับการเปรียบเทียบกับทางลัดสาธารณะ 1,118
ไฟล์ จาก 30 คอลัมน์ CSV ที่มีร่วมกับพาร์เซอร์อ้างอิง มีค่าที่แตกต่างกัน 16 ค่า ได้แก่
พาธเป้าหมาย 13 รายการที่เกี่ยวข้องกับ item ที่เสียหาย ถูกตัดทอน เป็น placeholder, MTP หรือ Control Panel
รายการ extra-data block สองรายการที่เกี่ยวข้องกับบล็อกที่ไม่รู้จักหรือถูกตัดทอน และ MFT reference หนึ่งรายการที่
พาร์เซอร์อ้างอิงอ่านจากไบต์อื่น Juntei รายงานโครงสร้างที่เสียหายแทนที่จะ
normalize โดยไม่แจ้ง

## Jump Lists { #jump-lists }

[ys-jump-lists-parser](https://github.com/Yamato-Security/ys-jump-lists-parser) ใช้งานด้าน
compound file และ Jump List ของ Sootmark โดยลิงก์ที่ฝังอยู่จะถูกถอดรหัสด้วย `ys-lnk-parser` จาก Jump Lists สาธารณะ
77 ไฟล์ เซลล์เปรียบเทียบที่มีร่วมกันทั้งหมด 13,057 เซลล์ รวมถึง timestamp ละเอียดถึง 100 ns ตรงกับ
พาร์เซอร์อ้างอิง ไฟล์หกไฟล์ที่พาร์เซอร์อ้างอิงปฏิเสธ (custom list ที่ว่าง ไม่มี footer หรือไฟล์
ที่ไม่ใช่ Jump List) ได้รับการถอดรหัสหรือรายงานอย่างชัดเจนโดย Juntei

## ถังรีไซเคิล { #recycle-bin }

`parse-file recycle-bin` เป็นพาร์เซอร์ที่ Juntei พัฒนาขึ้นเอง บนตัวอย่างสาธารณะของ
[rifiuti2](https://github.com/abelcheung/rifiuti2) ทุกเรคคอร์ดที่ rifiuti2 อ่านได้มีดัชนี เวลาที่ลบ (ละเอียดถึงวินาที
ตามที่ rifiuti2 แสดง) สถานะการนำออกจากถัง ขนาด และพาธตรงกัน รวม 92 เรคคอร์ด โดย 67 เรคคอร์ดมาจากไฟล์ INFO2
สิบเอ็ดไฟล์ของ Windows 95 ถึง Server 2003 และ 25 เรคคอร์ดมาจากสามโฟลเดอร์ที่มีไฟล์ `$I` ของ Windows Vista และ
Windows 10 พาธ ANSI ของ INFO2 ตรงกันใน code page ที่ชุดทดสอบของ rifiuti2 เองใช้ Juntei ยังอ่านสิ่งที่ rifiuti2
ปฏิเสธได้ด้วย ได้แก่ โฟลเดอร์ที่มีไฟล์ `$I` ทั้งสองเวอร์ชันปนกัน ส่วนหัว `$I` ที่ไม่มีพาธ และเรคคอร์ดที่ถูกตัดทอน
โดยรายงานความเสียหายของแต่ละรายการ ไฟล์ `$I` และ INFO2 ในข้อมูลทดสอบของ Plaso ให้เวลา ขนาด พาธ ดัชนี และไดรฟ์
ตามที่ชุดทดสอบของ Plaso คาดหวัง บนดิสก์อิมเมจ M57-Jean สาธารณะของ Digital Corpora (Windows XP) ซึ่งอ่านด้วย
`auto-parse --e01` เรคคอร์ด INFO2 ตรงกับ rifiuti2 และไฟล์ที่ถูกรีไซเคิลซึ่งอยู่ข้างกันคือ `Dc1.jpg` ก็มีชื่อตรงกับที่
Juntei ระบุ

## แคชบิตแมปของ RDP { #rdp-bitmap-cache }

`parse-file rdp-cache` เป็นพาร์เซอร์ที่ Juntei พัฒนาขึ้นเอง บน kofu ซึ่งเป็นหนึ่งในหกโฮสต์ของชุดข้อมูล CODE BLUE 2026
CTF ไทล์ทั้ง 1,911 ไทล์ในไฟล์ `Cache????.bin` สามไฟล์ของโฮสต์นี้ตรงกับภาพที่
[bmc-tools](https://github.com/ANSSI-FR/bmc-tools) สร้างขึ้นทุกพิกเซล ไทล์ 1,980 ไทล์ในไฟล์ `Cache0000.bin`
สาธารณะของโจทย์ "Phantom Connection" จาก ApoorvCTF 2025 ก็ตรงกันเช่นกัน โดย 105 ไทล์ในจำนวนนี้มีขนาด
64 × 48 พิกเซล bmc-tools อ่านทุกไทล์ว่ากว้าง 64 พิกเซล ไทล์ที่แคบกว่าซึ่งอยู่ที่ขอบขวาของหน้าจอจึงออกมาเบี้ยว
ส่วน Juntei อ่านแต่ละไทล์ตามความกว้างจริงของไทล์นั้น

ไม่พบแคช `.bmc` สาธารณะที่มีไทล์อยู่ ตัวอย่างข้างต้นเช่นเดียวกับไคลเอนต์ Windows 10 ขึ้นไปอื่น ๆ มีเพียงไฟล์
`bcache24.bmc` ที่ว่างเปล่า จึงทดสอบการอ่าน `.bmc` ด้วยไฟล์ที่สร้างขึ้นตามรูปแบบนี้ ได้แก่ ไทล์ที่ไม่ได้บีบอัด
ของความลึกสีทั้งสี่แบบ ไทล์ที่เล็กกว่าช่องและตามด้วยส่วนท้ายของไทล์ก่อนหน้า ช่องที่เสียหาย และไทล์ 16 บิตแบบสุ่ม
3,000 ไทล์ที่บีบอัดด้วย RLE ซึ่งใช้ order ครบทุกชนิดของ MS-RDPBCGR 3.1.9
โดยไม่มี order ใดยาวเลยท้ายแถวแรกของไทล์ Juntei และ bmc-tools ถอดรหัสทั้ง 3,000 ไทล์ได้พิกเซลเดียวกัน
เมื่อ order เริ่มต้นที่แถวแรกและยาวเลยแถวนั้นไป ทั้งสองจะให้ผลต่างกัน: Juntei ทำตามข้อกำหนดและตัวถอดรหัสของ
FreeRDP ซึ่งใช้กฎของแถวแรกตลอดทั้ง order ขณะที่ bmc-tools เปลี่ยนกฎกลางคัน

## ShimCache { #shimcache }

[ys-shimcache-parser](https://github.com/Yamato-Security/ys-shimcache-parser) เป็น fork ที่ดูแลต่อจาก
[shimcache-forensic](https://github.com/SecurityRonin/shimcache-forensic) ของ Security Ronin
พร้อมการรองรับไฮฟ์รีจิสทรีและบันทึกธุรกรรม บนไฮฟ์ `SYSTEM` สาธารณะ 20 ไฮฟ์ (24 control set และ 7,290 รายการ) รวมกับ
ค่า AppCompatCache แบบแยกเดี่ยว 17 ค่า ทุกรายการตรงกับพาธ เวลา แฟล็กการเรียกใช้งาน แฟล็ก และ
ตำแหน่งที่พาร์เซอร์อ้างอิงส่งออก

## การกักกันของ Microsoft Defender { #microsoft-defender-quarantine }

[ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser) เป็น
fork ที่ดูแลต่อจาก crate
[defender](https://github.com/Sootmark/defender) ของ Sootmark โฟลเดอร์ quarantine ที่เขียนในรูปแบบของ Defender
โดยเครื่องมือทดสอบของพาร์เซอร์ ถูกถอดรหัสได้อย่างสอดคล้องกันโดยตัวอ่าน quarantine ของ
`dissect.target` ที่เป็นอิสระ

## อาร์ติแฟกต์ที่ได้จากรีจิสทรี { #registry-derived-artifacts }

[ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser) ได้รับการเปรียบเทียบกับ
[yarp](https://github.com/msuhanov/yarp) สำหรับคีย์และค่าที่จัดสรรแล้ว การเล่นซ้ำธุรกรรมของไฮฟ์ที่อยู่ในสถานะ dirty
การกู้คืน free cell, cell slack, ข้อมูลที่อยู่เกิน hive bin และเรคคอร์ดที่ถูกลบหรือถูกแทนที่จากการเล่นซ้ำ
การเปรียบเทียบเฉพาะอาร์ติแฟกต์ได้แก่

- บริการ ค่า และ failure action จากไฮฟ์ `SYSTEM` ของ Windows 7 และ Windows 11 ที่อยู่ในสถานะ dirty
- RecentDocs ที่ถอดรหัสด้วย yarp และ libfwsi จากไฮฟ์ `NTUSER.DAT` สาธารณะเก้าไฮฟ์
- งานตามกำหนดเวลาที่ถอดรหัสโดยอิสระด้วย yarp จากไฮฟ์ `SOFTWARE` สาธารณะเจ็ดไฮฟ์ ครอบคลุม
  Windows 7–11, Server 2012 R2 และ Server 2022 และ
- โครงสร้างบัญชี SAM ที่เปรียบเทียบกับ `dissect.target` จากไฮฟ์ `SAM` สาธารณะเจ็ดไฮฟ์

ดู[กิตติกรรมประกาศ](../acknowledgements.md) สำหรับรายการเครื่องมืออ้างอิง งานวิจัยด้านรูปแบบไฟล์
ข้อกำหนด และแหล่งหลักฐานสาธารณะทั้งหมด
