# การเมานต์อิมเมจดิสก์ { #mounting-disk-images }

`juntei mount` ทำให้ไฟล์ภายในอิมเมจดิสก์ VHDX อ่านได้ผ่านไดเรกทอรีแบบอ่านอย่างเดียวขณะคำสั่งทำงาน เครื่องมืออย่าง Hayabusa และ Juntei สามารถตรวจสอบได้โดยไม่ต้องแยกไฟล์หรือใช้สิทธิ์ root ฟังก์ชันนี้ใช้ FUSE และทำงานบน Linux ตัวอ่านและไลบรารีการเมานต์ได้รับการดูแลใน [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser)

```bash
mkdir /mnt/host1
juntei mount host1-triage.vhdx /mnt/host1
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
- ชื่อจะแสดงตามที่ NTFS จัดเก็บไว้ และการค้นหาชื่อต้องตรงกันทุกตัวอักษร ไม่แสดงชื่อ DOS 8.3
  และ alternate data stream
- ไฟล์ที่ถูกบีบอัดจะถูกคลายการบีบอัด (สูงสุดไฟล์ละ 1 GiB) ไม่สามารถอ่านไฟล์ที่เข้ารหัสด้วย EFS ได้

## หลักฐานจะไม่ถูกเปลี่ยนแปลง { #the-evidence-is-not-changed }

อิมเมจถูกเปิดแบบอ่านอย่างเดียวและการเมานต์ปฏิเสธการเขียน หาก VHDX ไม่ได้ปิดอย่างสมบูรณ์ log ที่ค้างอยู่จะถูกนำมาใช้ในหน่วยความจำโดยไม่เขียนกลับลงอิมเมจ ตัวเขียน DiscUtils บางตัวปล่อยคำสั่งกระโดดและลายเซ็น `55 AA` ในบูตเซกเตอร์ NTFS เป็นศูนย์ Juntei อ่านวอลุ่มเหล่านี้ได้ตามสภาพเดิม

## การใช้งาน { #usage }

```text
Version: 1.1.0 (Dev Build)
Yamato Security (https://github.com/Yamato-Security/juntei - @SecurityYamato)

Usage:
  juntei mount <IMAGE> <MOUNTPOINT> [OPTIONS]

Input:
  <IMAGE>       The disk image (VHDX)
  <MOUNTPOINT>  An existing directory to mount its NTFS volumes at, read-only, until Ctrl+C

General Options:
  -h, --help  Show the help menu

Display Settings:
  -K, --no-color  Disable color output
  -q, --quiet     Quiet mode: do not display the launch banner
```

## การพาร์สทุกอิมเมจด้วย auto-parse { #parsing-every-image-with-auto-parse }

`auto-parse --vhdx` พาร์สระบบ Windows ภายในอิมเมจดิสก์ VHDX ทุกไฟล์ในไดเรกทอรีหลักฐาน (หรืออิมเมจเดียว
ที่ระบุด้วย `-d`) โดยไม่ต้องเมานต์แยกต่างหาก:

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
```

- แต่ละอิมเมจจะถูกเมานต์แบบอ่านอย่างเดียวในไดเรกทอรีชั่วคราวใหม่ ภายในโฟลเดอร์ที่ตั้งชื่อตามอิมเมจ
  (`host1-triage`) และระบบที่พบในนั้นจะถูกพาร์สเหมือนกับชุดข้อมูลที่อยู่บนดิสก์ อิมเมจจะถูกยกเลิกการเมานต์
  เมื่อทำงานเสร็จ และเมื่อกด Ctrl+C
- เทอร์มินัลแสดงรายการอิมเมจและอิมเมจของแต่ละระบบ:
  `HOST1: ./host1-triage/C (./host1-triage.vhdx)`
- ใน manifest `EvidenceDirectory` คือไดเรกทอรีที่ระบุ และ `Image` ของแต่ละระบบระบุอิมเมจของระบบนั้น
  `EvidenceRoot` และคำสั่งของพาร์เซอร์ใช้โฟลเดอร์เมานต์ (`./host1-triage/C`)
- หากอิมเมจมีไฟล์จาก Volume Shadow Copies ในโฟลเดอร์เช่น `VSS32` แต่ละโฟลเดอร์จะถูกตรวจพบและพาร์สเป็นอีกระบบหนึ่ง (`HOST1_2`, …)
- อิมเมจที่เมานต์ไม่ได้จะถูกรายงาน อิมเมจอื่น ๆ ยังคงถูกพาร์สต่อ และการทำงานจะจบลงด้วย exit status 1

หากเปิดโวลุ่ม NTFS ไม่ได้ Juntei จะรายงานปัญหาและคงโวลุ่มที่อ่านได้ไว้ สำหรับอิมเมจที่อ่านได้เพียงบางส่วน `auto-parse --vhdx` จะคืนรหัสจบการทำงาน 1 และ `mount` จะคืนค่า 1 หลังยกเลิกการเมานต์เช่นกัน

## สิ่งที่ต้องมี { #requirements }

- Linux: FUSE 3 (`fusermount3`, แพ็กเกจ `fuse3`) และสิทธิ์เข้าถึง `/dev/fuse` ซึ่งผู้ใช้ทั่วไปมักมีอยู่แล้ว
  ไม่จำเป็นต้องใช้ root
- ไม่รองรับ Windows และ macOS บน Windows ให้แนบ (attach) อิมเมจแบบอ่านอย่างเดียวใน PowerShell ที่รันด้วยสิทธิ์ผู้ดูแลระบบแทน
  แล้วระบุอักษรไดรฟ์ของอิมเมจนั้นให้กับเครื่องมือ:

```powershell
Mount-DiskImage -ImagePath D:\cases\host1-triage.vhdx -Access ReadOnly
juntei auto-parse -d E:\ -o host1-juntei
Dismount-DiskImage -ImagePath D:\cases\host1-triage.vhdx
```
