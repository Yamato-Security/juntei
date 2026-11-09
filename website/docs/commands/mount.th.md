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

`auto-parse --vhdx` อ่าน VHDX แบบเนทีฟบน **Linux, macOS และ Windows** โดยไม่ต้องเมานต์ผ่าน OS ใช้ FUSE หรือมีสิทธิ์ผู้ดูแลระบบ

```bash
juntei auto-parse -d /cases/vhdx --vhdx -o parsed
juntei auto-parse -d host1.vhdx --vhdx -o parsed -j timeline.jsonl
```

พาร์เซอร์ต้องใช้พาธไฟล์ Juntei จึงคัดลอกไฟล์ที่ตัวอ่าน NTFS แสดงไปยังไดเรกทอรีชั่วคราวส่วนตัว **ต้องมีพื้นที่ดิสก์ชั่วคราวเพียงพอสำหรับไฟล์เหล่านี้ทั้งหมด** อิมเมจต้นฉบับยังคงอ่านอย่างเดียว และสำเนาจะถูกลบเมื่อจบการทำงานตามปกติ

ข้อผิดพลาดในการอ่านและชื่อที่ไม่ปลอดภัยจะบันทึกใน `ImageErrors` และแถวที่ล้มเหลวของ `manifest.csv` ผลลัพธ์ที่อ่านได้ยังคงถูกเก็บไว้ และอินพุตที่ไม่สมบูรณ์จะคืนรหัสออก 1

คำสั่ง `mount` แยกต่างหากสำหรับให้โปรแกรมอื่นเข้าถึงไฟล์ยังคงต้องใช้ Linux, FUSE 3 และสิทธิ์เข้าถึง `/dev/fuse`

[English: implementation details and limitations](https://yamato-security.github.io/juntei/commands/auto-parse/#vhdx-disk-images).
