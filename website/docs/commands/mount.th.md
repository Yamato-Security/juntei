# การเมานต์อิมเมจดิสก์ { #mounting-disk-images }

`juntei mount` ทำให้ไฟล์ภายในอิมเมจดิสก์ VHDX อ่านได้ที่ไดเรกทอรีหนึ่งแบบอ่านอย่างเดียว ตลอดเวลาที่คำสั่งทำงานอยู่
จากนั้นชุดข้อมูลที่เก็บด้วย `--vhdx` ของ KAPE และดิสก์ของ Hyper-V ก็สามารถสแกนได้ด้วยเครื่องมือใดก็ได้ เช่น Hayabusa
หรือคำสั่งของ Juntei เอง โดยไม่ต้องแยกไฟล์ใด ๆ ออกมา ไม่ต้องใช้สิทธิ์ root และไม่ต้องให้ระบบปฏิบัติการรองรับ VHDX
หรือ NTFS คำสั่งนี้ใช้ FUSE และทำงานบน Linux

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

- ไฟล์ของวอลุ่ม NTFS ที่จุดเมานต์ (mount point): สำหรับชุดข้อมูลของ KAPE ได้แก่ โฟลเดอร์ `C` ที่เก็บไฟล์
  ที่รวบรวมมา โฟลเดอร์ `LongFileNames` ของ KAPE และไฟล์เมทาดาทาของวอลุ่มเอง เช่น `$MFT` อิมเมจที่มีวอลุ่ม
  NTFS หลายวอลุ่มจะแสดงวอลุ่มเหล่านั้นเป็น `Volume1`, `Volume2` และต่อ ๆ ไป
- ทุกไฟล์เป็นแบบอ่านอย่างเดียวและเป็นของเจ้าของจุดเมานต์ เวลาของไฟล์คือเวลาใน `$STANDARD_INFORMATION`
  ของ NTFS
- ชื่อจะแสดงตามที่ NTFS จัดเก็บไว้ และการค้นหาชื่อต้องตรงกันทุกตัวอักษร ไม่แสดงชื่อ DOS 8.3
  และ alternate data stream
- ไฟล์ที่ถูกบีบอัดจะถูกคลายการบีบอัด (สูงสุดไฟล์ละ 1 GiB) ไม่สามารถอ่านไฟล์ที่เข้ารหัสด้วย EFS ได้

## หลักฐานจะไม่ถูกเปลี่ยนแปลง { #the-evidence-is-not-changed }

อิมเมจจะถูกเปิดแบบอ่านอย่างเดียว และการเมานต์จะปฏิเสธการเขียนทุกรูปแบบ VHDX ที่ไม่ได้ถูกปิดอย่างเรียบร้อยจะถูกอ่าน
โดยนำ log ที่ค้างอยู่มาใช้ในหน่วยความจำ และจะไม่มีการเขียนกลับลงไฟล์เลย KAPE เขียนไฟล์ VHDX ด้วย DiscUtils
ซึ่งทำให้บูตเซกเตอร์ของ NTFS ไม่มีคำสั่ง jump และลายเซ็น `55 AA` แต่ Juntei อ่านวอลุ่มเหล่านี้ได้ตามสภาพที่เป็นอยู่

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
- KAPE เก็บไฟล์ที่คัดลอกมาจาก Volume Shadow Copies ไว้ในโฟลเดอร์ เช่น `VSS32` แต่ละโฟลเดอร์จะถูกค้นพบ
  และพาร์สเป็นอีกระบบหนึ่ง (`HOST1_2`, …)
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
