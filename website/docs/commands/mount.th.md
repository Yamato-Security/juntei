# การเมานต์อิมเมจดิสก์ { #mounting-disk-images }

`juntei mount` ทำให้ไฟล์ภายในอิมเมจดิสก์ (อิมเมจ VHDX หรือ E01) อ่านได้ผ่านไดเรกทอรีแบบอ่านอย่างเดียวขณะคำสั่งทำงาน เครื่องมืออย่าง Hayabusa และ Juntei สามารถตรวจสอบได้โดยไม่ต้องแยกไฟล์หรือใช้สิทธิ์ root ฟังก์ชันนี้ใช้ FUSE และทำงานบน Linux ตัวอ่าน VHDX และไลบรารีการเมานต์ได้รับการดูแลใน [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) ส่วนตัวอ่าน E01 ได้รับการดูแลใน [ys-e01-parser](https://github.com/Yamato-Security/ys-e01-parser)

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
```

อิมเมจ E01 (Expert Witness Format ของ EnCase รวมถึง `.Ex01`) เมานต์ได้ด้วยวิธีเดียวกัน สำหรับอิมเมจที่แบ่งเป็นหลายเซกเมนต์ ให้ระบุเซกเมนต์แรก แล้ว Juntei จะค้นหาเซกเมนต์อื่น (`host1.E02`, `host1.E03`, …) ที่อยู่ในตำแหน่งเดียวกัน:

```bash
juntei mount host1.E01 /mnt/host1
```

ในเทอร์มินัลอื่น:

```bash
hayabusa dfir-timeline -d /mnt/host1 -o host1-hayabusa.csv
juntei auto-parse -d /mnt/host1 -o host1-juntei
```

กด Ctrl+C ในเทอร์มินัลแรกเพื่อยกเลิกการเมานต์ หรือเรียกใช้ `fusermount3 -u /mnt/host1`

## สิ่งที่การเมานต์แสดง { #what-the-mount-shows }

- ไฟล์ของวอลุ่ม NTFS จะปรากฏที่จุดเมานต์ อิมเมจที่รวบรวมอาจมีไดเรกทอรีเช่น `C` และ `LongFileNames` พร้อมเมทาดาทาวอลุ่มเช่น `$MFT` หากมีหลายวอลุ่ม NTFS จะแสดงเป็น `Volume1`, `Volume2` เป็นต้น
- ทุกไฟล์เป็นแบบอ่านอย่างเดียวและเป็นของเจ้าของจุดเมานต์ เวลาของไฟล์คือเวลาใน `$STANDARD_INFORMATION`
  ของ NTFS
- อิมเมจของวอลุ่มเดียวที่ไม่มีตารางพาร์ทิชันจะถือเป็นวอลุ่มนั้นเอง
- ชื่อจะแสดงตามที่ NTFS จัดเก็บไว้ และการค้นหาชื่อต้องตรงกันทุกตัวอักษร ไม่แสดงชื่อ DOS 8.3
  และ alternate data stream
- ไฟล์ที่ถูกบีบอัดจะถูกคลายการบีบอัด (สูงสุดไฟล์ละ 1 GiB) ไม่สามารถอ่านไฟล์ที่เข้ารหัสด้วย EFS ได้

## หลักฐานจะไม่ถูกเปลี่ยนแปลง { #the-evidence-is-not-changed }

อิมเมจถูกเปิดแบบอ่านอย่างเดียวและการเมานต์ปฏิเสธการเขียน หาก VHDX ไม่ได้ปิดอย่างสมบูรณ์ log ที่ค้างอยู่จะถูกนำมาใช้ในหน่วยความจำโดยไม่เขียนกลับลงอิมเมจ ตัวเขียน DiscUtils บางตัวปล่อยคำสั่งกระโดดและลายเซ็น `55 AA` ในบูตเซกเตอร์ NTFS เป็นศูนย์ Juntei อ่านวอลุ่มเหล่านี้ได้ตามสภาพเดิม

เซกเมนต์ของอิมเมจ E01 ก็ถูกเปิดแบบอ่านอย่างเดียวเช่นกัน แต่ละ chunk จะถูกตรวจสอบกับ checksum ของตนเมื่ออ่าน: chunk ที่เสียหายจะอ่านไม่ได้ แต่ส่วนที่เหลือของอิมเมจยังอ่านได้ เซกเตอร์ที่อ่านไม่ได้ระหว่างการเก็บหลักฐาน และการเก็บหลักฐานที่ไม่เสร็จสมบูรณ์ จะถูกรายงานเป็นคำเตือน ไฟล์หลักฐานเชิงตรรกะ (`.L01`, `.Lx01`) เก็บไฟล์ ไม่ใช่ดิสก์ จึงไม่ถูกเมานต์

## การใช้งาน { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX or E01; a split E01 image from its first segment)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## การพาร์สทุกอิมเมจด้วย auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` พาร์สระบบ Windows ภายในอิมเมจดิสก์ VHDX ทุกไฟล์ในไดเรกทอรีหลักฐาน (หรืออิมเมจเดียว
ที่ระบุด้วย `-d`) และ `auto-parse --e01` พาร์สระบบ Windows ภายในอิมเมจ E01 ทุกไฟล์ โดยไม่ต้องเมานต์แยกต่างหาก
สามารถระบุทั้งสองตัวเลือกพร้อมกันได้:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d /cases/e01 --e01 -o parsed
```

- แต่ละอิมเมจจะถูกเมานต์แบบอ่านอย่างเดียวในไดเรกทอรีชั่วคราวใหม่ ภายในโฟลเดอร์ที่ตั้งชื่อตามอิมเมจ
  (`host1-triage`) และระบบที่พบในนั้นจะถูกพาร์สเหมือนกับชุดข้อมูลที่อยู่บนดิสก์ อิมเมจจะถูกยกเลิกการเมานต์
  เมื่อทำงานเสร็จ และเมื่อกด Ctrl+C
- เทอร์มินัลแสดงรายการอิมเมจและอิมเมจของแต่ละระบบ:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`
- `--e01` รับเซกเมนต์แรกของอิมเมจ E01 แต่ละไฟล์ (`*.E01`, `*.Ex01`) ส่วนเซกเมนต์อื่นจะถูกอ่านไปพร้อมกัน
- ใน manifest `EvidenceDirectory` คือไดเรกทอรีที่ระบุ และ `Image` ของแต่ละระบบระบุอิมเมจของระบบนั้น
  `EvidenceRoot` และคำสั่งของพาร์เซอร์ใช้โฟลเดอร์เมานต์ (`./host1-triage/C`)
- หากอิมเมจมีไฟล์จาก Volume Shadow Copies ในโฟลเดอร์เช่น `VSS32` แต่ละโฟลเดอร์จะถูกตรวจพบและพาร์สเป็นอีกระบบหนึ่ง (`HOST1_2`, …)
- อิมเมจที่เมานต์ไม่ได้จะถูกรายงาน อิมเมจอื่น ๆ ยังคงถูกพาร์สต่อ และการทำงานจะจบลงด้วย exit status 1

หากเปิดโวลุ่ม NTFS ไม่ได้ Juntei จะรายงานปัญหาและคงโวลุ่มที่อ่านได้ไว้ สำหรับอิมเมจที่อ่านได้เพียงบางส่วน `auto-parse --vhdx` และ `--e01` จะคืนรหัสจบการทำงาน 1 และ `mount` จะคืนค่า 1 หลังยกเลิกการเมานต์เช่นกัน

## สิ่งที่ต้องมี { #requirements }

- Linux: FUSE 3 (`fusermount3`, แพ็กเกจ `fuse3`) และสิทธิ์เข้าถึง `/dev/fuse` ซึ่งผู้ใช้ทั่วไปมักมีอยู่แล้ว
  ไม่จำเป็นต้องใช้ root
- ไม่รองรับ Windows และ macOS บน Windows ให้แนบ (attach) อิมเมจ VHDX แบบอ่านอย่างเดียวใน PowerShell ที่รันด้วยสิทธิ์ผู้ดูแลระบบแทน
  แล้วระบุอักษรไดรฟ์ของอิมเมจนั้นให้กับเครื่องมือ Windows แนบอิมเมจ E01 ไม่ได้: ให้เมานต์ด้วยเครื่องมือที่ทำได้
  แบบอ่านอย่างเดียว แล้วระบุไดรฟ์นั้นให้กับเครื่องมือ

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
