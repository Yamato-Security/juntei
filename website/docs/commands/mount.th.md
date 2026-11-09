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

## การวิเคราะห์อิมเมจแบบเนทีฟ { #parsing-every-image-with-auto-parse }

เมื่อใช้ `--vhdx` คำสั่ง `auto-parse` อ่าน VHDX แบบเนทีฟบน **Linux, macOS และ Windows** ผ่าน [ys-vhdx-parser](https://github.com/Yamato-Security/ys-vhdx-parser) โดยไม่ต้องเมานต์ OS ใช้ FUSE หรือมีสิทธิ์ผู้ดูแลระบบ

```bash
# A directory of images, searched recursively
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
# One image
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

พาร์เซอร์ต้องใช้พาธ Juntei จึงคัดลอกไฟล์ NTFS ที่แสดงไปยังพื้นที่ชั่วคราวส่วนตัวด้วยบัฟเฟอร์จำกัด **ต้องมีพื้นที่สำหรับไฟล์เหล่านี้ทั้งหมด** ไม่ใช่เฉพาะอาร์ติแฟกต์ที่ให้ผลลัพธ์ หากรองรับจะเขียนพื้นที่ศูนย์เป็นพื้นที่ sparse โดยรักษาชื่อ โครงสร้าง ฮาร์ดลิงก์ และเวลาแก้ไข เวลาแหล่งข้อมูล LNK และ Jump List มาจากสแนปช็อตเมทาดาทา NTFS เดิม ไม่ใช่สำเนาชั่วคราว เมื่อจบตามปกติรวมถึงกรณีพาร์เซอร์ผิดพลาดจะลบสำเนา แต่การบังคับหยุดอาจทิ้งไฟล์ไว้ ต้นฉบับอ่านอย่างเดียวและใช้ล็อก VHDX ที่ค้างในหน่วยความจำเท่านั้น

การค้นหาโฮสต์ CSV/JSON/JSONL และ `-j` ใช้ขั้นตอนปกติ `Image` ใน `manifest.json` ระบุอิมเมจต้นฉบับ ส่วน `EvidenceRoot` ระบุพาธสัมพัทธ์ภายใน ข้อผิดพลาดการอ่านและชื่อไม่ปลอดภัยอยู่ใน `ImageErrors` และแถวล้มเหลวของ `manifest.csv` จะเก็บผลที่อ่านได้และคืนรหัส 1 หากอินพุตไม่สมบูรณ์ ไม่เปลี่ยนชื่ออันตรายหรือกำกวม และไม่แยกสตรีม NTFS ที่มีชื่อเป็นไฟล์เพิ่มเติม อาร์ติแฟกต์ที่รวบรวมเป็นไฟล์ปกติยังใช้ได้

คำสั่ง [`mount` แยกต่างหาก](mount.md) เปิดระบบไฟล์ให้โปรแกรมอื่นและยังต้องใช้ Linux FUSE โดย `auto-parse --vhdx` ไม่ได้เรียกใช้คำสั่งนี้

พื้นที่เตรียมข้อมูลเริ่มต้นเปลี่ยนเป็นระบบไฟล์ของผลลัพธ์ แทนไดเรกทอรีชั่วคราวของระบบซึ่งอาจเป็นวอลุ่ม RAM ขนาดเล็ก ใช้ `--temp-dir` เพื่อเลือกไดเรกทอรีที่มีอยู่บนดิสก์อื่น เมทาดาทาต้นฉบับเก็บแยกบนระบบไฟล์ผลลัพธ์ จึงยังแยกวิเคราะห์ไฟล์ที่อ่านได้แล้วเมื่อดิสก์เตรียมข้อมูลเต็ม ควรสำรองพื้นที่สำหรับทั้งสำเนาและผลลัพธ์ หากวอลุ่มผลลัพธ์เต็มจะบันทึกผลไม่ได้ Ctrl+C หยุดโปรเซสตัวแยกวิเคราะห์ ล้างพื้นที่ส่วนตัว และออกด้วยรหัส 130 แต่ SIGKILL หรือไฟดับยังอาจทิ้งสำเนาไว้ การค้นหาอิมเมจจะข้ามไดเรกทอรีย่อยที่อ่านไม่ได้ แต่รากอินพุตที่อ่านไม่ได้ยังเป็นข้อผิดพลาด

```bash
juntei auto-parse --vhdx -d evidence.vhdx -o parsed --temp-dir /large-disk/work
```

## ข้อกำหนดสำหรับการเมานต์ที่ OS มองเห็น { #os-visible-mount-requirements }

`juntei mount` ต้องใช้ Linux FUSE 3 (`fusermount3`) และเข้าถึง `/dev/fuse` ได้ บน Windows และ macOS ให้ใช้การวิเคราะห์แบบเนทีฟด้านบน
