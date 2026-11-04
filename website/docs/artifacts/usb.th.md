# อุปกรณ์ USB { #usb-devices }

## ประวัติ USB คืออะไร { #what-the-usb-history-is }

Windows บันทึกทุกอุปกรณ์ที่ตนติดตั้ง สำหรับอุปกรณ์จัดเก็บข้อมูล USB อุปกรณ์ USB อื่น ๆ และอุปกรณ์พกพา
(โทรศัพท์ กล้อง เครื่องเล่นสื่อ) ไฮฟ์ `SYSTEM` จะเก็บคีย์ของแต่ละอุปกรณ์ไว้พร้อมชื่อ หมายเลขซีเรียล
และเวลาที่ Windows ติดตั้งอุปกรณ์ เวลาที่เห็นอุปกรณ์เชื่อมต่อเข้ามาครั้งล่าสุด และเวลาที่เห็นอุปกรณ์ถูกถอดออกครั้งล่าสุด
`MountedDevices` บันทึก volume GUID และอักษรไดรฟ์ที่กำหนดให้กับวอลุ่มของอุปกรณ์เหล่านั้น ไฮฟ์
`SOFTWARE` เก็บชื่อที่ Explorer ตั้งให้กับอุปกรณ์พกพาและวอลุ่ม รวมถึงบันทึก serial ของวอลุ่มของ ReadyBoost
ส่วน `NTUSER.DAT` ของผู้ใช้แต่ละรายจะบันทึกวอลุ่มที่ผู้ใช้นั้นเคยเมานต์

`parse-registry usb` รวมข้อมูลเหล่านี้เป็นเรคคอร์ดเดียวต่ออุปกรณ์และวอลุ่ม ได้แก่ อุปกรณ์นั้นคืออะไร
ถูกติดตั้งและเชื่อมต่อเมื่อใด มีอักษรไดรฟ์และวอลุ่มใด และผู้ใช้รายใดเคยเมานต์วอลุ่มนั้น

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> อุปกรณ์ที่มีผู้ผลิต ผลิตภัณฑ์ และหมายเลขซีเรียลนี้
เคยเชื่อมต่อกับระบบนี้ ครั้งแรกและครั้งล่าสุด ณ เวลาเหล่านี้ วอลุ่มของอุปกรณ์มีอักษรไดรฟ์นี้ และผู้ใช้เหล่านี้เคยเมานต์วอลุ่มนั้น
เรคคอร์ดในรีจิสทรีไม่ได้แสดงว่ามีการคัดลอกไฟล์ใด

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SYSTEM
  ControlSet00N\Enum\USBSTOR\<Disk&Ven_…&Prod_…&Rev_…>\<serial>&<LUN>
  ControlSet00N\Enum\USB\<VID_xxxx&PID_yyyy>\<serial>
  ControlSet00N\Enum\SWD\WPDBUSENUM\<device path>
    Properties\{83da6326-97a6-4088-9453-a1923f573b29}\0064 … 0067
  MountedDevices
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows Portable Devices\Devices
  Microsoft\Windows NT\CurrentVersion\EMDMgmt
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Explorer\MountPoints2
```

| คุณสมบัติของอุปกรณ์ | ฟิลด์ | ความหมาย (`devpkey.h`) |
|---|---|---|
| `0064` | `DriverLastInstalledUTC` | `DEVPKEY_Device_InstallDate`: เวลาที่ติดตั้งไดรเวอร์ของอุปกรณ์ครั้งล่าสุด |
| `0065` | `DeviceFirstInstallUTC` | `DEVPKEY_Device_FirstInstallDate`: เวลาที่ติดตั้งอุปกรณ์ครั้งแรก |
| `0066` | `LastArrivalUTC` | `DEVPKEY_Device_LastArrivalDate`: เวลาที่เชื่อมต่อครั้งล่าสุด (Windows 8 ขึ้นไป) |
| `0067` | `LastRemovalUTC` | `DEVPKEY_Device_LastRemovalDate`: เวลาที่ถอดออกครั้งล่าสุด (Windows 8 ขึ้นไป) |

Windows 8 ขึ้นไปเก็บแต่ละเวลาไว้เป็นค่าเริ่มต้น (default value) ของคีย์ `0064` … `0067` ส่วน Windows 7
เก็บไว้ใน `00000064\00000000\Data` โดย Juntei อ่านได้ทั้งสองแบบ

แหล่งอ้างอิง DFIR จำนวนมากระบุว่า `0064` คือการติดตั้งครั้งแรก และ `0065` คือการติดตั้ง Juntei ยึดตามนิยามใน
`devpkey.h` ของ Microsoft ข้างต้น และตั้งชื่อคอลัมน์ตามสิ่งที่แต่ละเวลาบันทึกไว้

ให้ส่งไฮฟ์ `SYSTEM` ให้กับ `-f` หรือ `-d` ไฮฟ์ `SOFTWARE` ของระบบและไฮฟ์ `NTUSER.DAT` ของผู้ใช้
จะถูกค้นหาจากตำแหน่งข้างเคียงตามโครงสร้างที่การเก็บรวบรวมเก็บรักษาไว้ (`Windows\System32\config\SOFTWARE`,
`Users\<user>\NTUSER.DAT`) หรือในไดเรกทอรีเดียวกันและลึกลงไปหนึ่งระดับเมื่อไฮฟ์ถูกส่งออกมาพร้อมกัน
ให้เก็บรวบรวมไฮฟ์ทั้งหมดพร้อม transaction log

## วิธีรวมเรคคอร์ด { #how-the-records-are-joined }

- **อุปกรณ์:** หนึ่งเรคคอร์ดต่อคีย์อุปกรณ์หนึ่งคีย์ ได้แก่ `usbstor`, `usb` และ `wpdbusenum` (`DeviceClass`)
  อุปกรณ์จัดเก็บข้อมูล USB จะได้ vendor ID และ product ID จากอุปกรณ์ USB ที่มี container ID เดียวกัน
  (หรือหมายเลขซีเรียลเดียวกันหากไม่มี)
- **วอลุ่ม:** วอลุ่มของอุปกรณ์จัดเก็บข้อมูล USB คือค่า `\??\Volume{GUID}` ของ
  `MountedDevices` ที่มีข้อมูลเป็นพาธซึ่งระบุถึงอุปกรณ์นั้น อุปกรณ์ที่มีสองวอลุ่มจะมีสองเรคคอร์ด
  อักษรไดรฟ์ของอุปกรณ์คือค่า `\DosDevices\X:` ที่มีข้อมูลเดียวกัน
- **ชื่อและหมายเลขซีเรียล:** ชื่อวอลุ่มมาจาก `Windows Portable Devices` และจากอุปกรณ์พกพา
  ที่ระบุถึงอุปกรณ์เดียวกัน ส่วนป้ายชื่อวอลุ่มและหมายเลขซีเรียลมาจากคีย์ `EMDMgmt` ที่ระบุถึงอุปกรณ์นั้น
- **ผู้ใช้:** ผู้ใช้ที่ `MountPoints2` ของตนมี GUID ของวอลุ่มนั้น พร้อมเวลาเขียนล่าสุดของคีย์ดังกล่าว
- **วอลุ่มที่ไม่มีคีย์อุปกรณ์** (`volume`): วอลุ่มใน `MountedDevices` ที่ไม่มีคีย์อุปกรณ์ใดระบุถึง (ของดิสก์ภายใน
  หรือของอุปกรณ์ที่ถูกถอดออกแล้วซึ่งคีย์ `Enum` หายไป) อักษรไดรฟ์ที่ไม่มีค่า volume GUID ใดมีข้อมูลตรงกัน
  และวอลุ่มที่มีอยู่เฉพาะใน `MountPoints2` ของผู้ใช้
- **อุปกรณ์ที่มีเฉพาะ `SOFTWARE` ระบุถึง** (`portable_device`, `readyboost`)

`SourceKeys` แสดงรายการทุกคีย์และค่าที่ใช้รวมเป็นเรคคอร์ดนั้น พร้อมไฮฟ์ของแต่ละรายการ

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **อักษรไดรฟ์ถูกนำกลับมาใช้ซ้ำ** `MountedDevices` เก็บเฉพาะอุปกรณ์ล่าสุดที่ได้รับแต่ละอักษร
  อักษรไดรฟ์จะแสดงเฉพาะบนอุปกรณ์ที่ข้อมูลยังคงเป็นของอักษรนั้นอยู่
- **หมายเลขซีเรียลอาจถูกสร้างขึ้นเอง** เมื่ออักขระตัวที่สองของ instance เป็น `&` แสดงว่า Windows
  สร้างขึ้นเองเพราะอุปกรณ์ไม่รายงานหมายเลขซีเรียล ในกรณีนี้ `Serial` จะว่างเปล่า instance ดังกล่าวอาจ
  แตกต่างกันบนคอมพิวเตอร์เครื่องอื่น
- **instance ของอุปกรณ์จัดเก็บข้อมูล USB ลงท้ายด้วย logical unit** (`&0`) `Serial` ตัดส่วนนี้ออก ส่วน `Instance`
  คงไว้
- **เวลาแตกต่างกันตามเวอร์ชันของ Windows** Windows 7 มีเพียงเวลาการติดตั้ง `LastArrivalUTC` และ
  `LastRemovalUTC` เป็นเพียงการเชื่อมต่อครั้งล่าสุด ไม่ใช่ทุกการเชื่อมต่อ event log
  (`Microsoft-Windows-Partition/Diagnostic` 1006, Kernel-PnP, `DriverFrameworks-UserMode`) และ
  `setupapi.dev.log` ให้ข้อมูลได้มากกว่า
- **control set:** โดยค่าเริ่มต้นจะอ่าน control set ที่ Windows ใช้ล่าสุด `--all-control-sets`
  อ่านทุก control set โดย control set ที่เก่ากว่าอาจยังแสดงอุปกรณ์ที่ชุดปัจจุบันไม่มีแล้ว
- **เวลาเขียนล่าสุดของคีย์ไม่ใช่เวลาเชื่อมต่อ** Windows เขียนคีย์อุปกรณ์ใหม่ด้วยเหตุผลหลายประการ
- **อุปกรณ์ที่ถูกลบออกยังทิ้งร่องรอยไว้** เรคคอร์ดวอลุ่มที่มีผู้ผลิตและผลิตภัณฑ์แต่ไม่มีคีย์อุปกรณ์
  คืออุปกรณ์ที่คีย์ `Enum` ถูกลบออก (จากการถอนการติดตั้งหรือเครื่องมือทำความสะอาด) แต่ `MountedDevices`
  ยังคงมีวอลุ่มของอุปกรณ์นั้นอยู่
- **หมายเลขซีเรียลของ ReadyBoost** คือหมายเลขซีเรียลของวอลุ่ม (`VolumeSerials` ตามที่ `dir` แสดง)
  ซึ่งไฟล์ LNK และ Jump Lists ก็บันทึกไว้เช่นกัน

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry usb <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives (with their SOFTWARE and NTUSER.DAT hives)
  -f, --file <FILE>      Parse one SYSTEM hive (with its SOFTWARE and NTUSER.DAT hives)
      --log1/--log2      Supply the SYSTEM hive's transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Read every control set

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry usb -d evidence/ -o usb.csv
juntei parse-registry usb -f C/Windows/System32/config/SYSTEM -t jsonl \
  | jq 'select(.DeviceClass == "usbstor") | {Vendor, Product, Serial, LastArrival, DriveLetters, Users}'
```

สรุปผลจะแสดงรายการอุปกรณ์จัดเก็บข้อมูล USB ที่เชื่อมต่อล่าสุด (ใหม่สุดก่อน) พร้อมอักษรไดรฟ์
และจำนวนวอลุ่มที่เป็นของอุปกรณ์จัดเก็บข้อมูล USB ที่ไม่มีคีย์อุปกรณ์

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host` | ชื่อคอมพิวเตอร์ จาก `SYSTEM` |
| `DeviceClass` | `usbstor`, `usb`, `wpdbusenum` หรือเมื่อไม่มีคีย์อุปกรณ์จะเป็น `volume`, `portable_device` หรือ `readyboost` |
| `ControlSet` | control set ของคีย์อุปกรณ์ |
| `Vendor`, `Product`, `Revision` | อุปกรณ์จัดเก็บข้อมูล USB: มาจากชื่อของอุปกรณ์ (`Disk&Ven_…&Prod_…&Rev_…`) รวมถึงอุปกรณ์ที่วอลุ่มหรือคีย์ `SOFTWARE` ระบุถึงด้วย |
| `Vid`, `PID` | vendor ID และ product ID ของ USB |
| `Serial`, `Instance` | หมายเลขซีเรียล และชื่อของคีย์ instance ตามที่บันทึกไว้ |
| `FriendlyName`, `DeviceDesc`, `Service`, `ContainerID` | ค่าของคีย์อุปกรณ์ตามที่บันทึกไว้ (`DeviceDesc` มักอยู่ในรูป `@<inf>,%…%;<text>`) `FriendlyName` ของอุปกรณ์พกพามักเป็นป้ายชื่อหรือไดรฟ์ของวอลุ่ม |
| `DeviceFirstInstallUTC`, `DriverLastInstalledUTC`, `LastArrivalUTC`, `LastRemovalUTC` | เวลาจากคุณสมบัติของอุปกรณ์ (ดูด้านบน) |
| `DriveLetters` | อักษรไดรฟ์ที่ข้อมูลใน `MountedDevices` เป็นของวอลุ่มนั้น |
| `VolumeGUID`, `VolumeData` | GUID ของวอลุ่ม และสิ่งที่ข้อมูลใน `MountedDevices` ระบุว่าวอลุ่มอยู่บนอะไร ได้แก่ พาธอุปกรณ์ ลายเซ็นดิสก์ MBR และ offset ของพาร์ทิชัน หรือพาร์ทิชัน GPT |
| `VolumeNames`, `VolumeSerials` | ชื่อที่ Explorer ตั้งให้กับอุปกรณ์หรือวอลุ่ม ป้ายชื่อ ReadyBoost และหมายเลขซีเรียลของวอลุ่ม (`1234-ABCD`) |
| `User` | ผู้ใช้ที่ `MountPoints2` ของตนมีวอลุ่มนั้น พร้อมเวลาเขียนล่าสุดของคีย์ดังกล่าว (JSON: `User`, `MountPoints2LastWritten`, `SourcePath`) |
| `FoundIn`, `SourceKeys` | ตำแหน่งที่พบ (`Enum`, `MountedDevices`, `Windows Portable Devices`, `EMDMgmt`, `MountPoints2`) และทุกคีย์และค่าที่ใช้รวมเป็นเรคคอร์ด |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์ instance ของอุปกรณ์และเวลาเขียนล่าสุด |
| `SourcePath`, `SoftwarePath` | ไฮฟ์ `SYSTEM` และไฮฟ์ `SOFTWARE` เมื่อไฮฟ์นั้นให้ข้อมูลบางอย่าง |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรีของไฮฟ์ที่ใช้รวมเป็นเรคคอร์ด |
| `Problems` | คีย์ที่อ่านไม่ได้ และคุณสมบัติของอุปกรณ์ที่ไม่ใช่ FILETIME (พร้อมไบต์ของค่านั้น) |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงลำดับอุปกรณ์จัดเก็บข้อมูล USB ตาม `LastArrivalUTC` (หรือ `DriverLastInstalledUTC` หากไม่มี) และแยก
   เครื่องอ่านการ์ดและฮับของคอมพิวเตอร์เองออก
2. จดบันทึกผู้ผลิต ผลิตภัณฑ์ หมายเลขซีเรียล volume GUID อักษรไดรฟ์ และหมายเลขซีเรียลของวอลุ่ม
   ของแต่ละอุปกรณ์ แล้วค้นหาค่าเหล่านี้ในไฟล์ LNK, Jump Lists, ShellBags และ `$MFT` (พาธบนอักษรไดรฟ์หรือวอลุ่มนั้น)
3. ตรวจสอบ `User` เพื่อดูว่าใครเคยเมานต์วอลุ่มนั้นและเมื่อใด (เวลาเขียนล่าสุดของคีย์ `MountPoints2`)
4. ค้นหาเรคคอร์ด `volume` ที่ระบุถึงอุปกรณ์จัดเก็บข้อมูล USB ซึ่งคืออุปกรณ์ที่คีย์ถูกลบออกไปแล้ว
5. เชื่อมโยงกับ event log เพื่อดูทุกการเชื่อมต่อ ไม่ใช่เฉพาะครั้งล่าสุด
6. ใช้ `parse-registry read --recover-deleted` สำหรับคีย์อุปกรณ์ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: คุณสมบัติของอุปกรณ์ (devpkey.h)](https://learn.microsoft.com/en-us/windows-hardware/drivers/install/devpkey-device-installdate)
- [Microsoft: การรองรับคำขอของ mount manager](https://learn.microsoft.com/en-us/windows-hardware/drivers/storage/supporting-mount-manager-requests-in-a-storage-class-driver)
- [`devpkey.h` (ตามที่อยู่ใน header ของ Wine)](https://github.com/wine-mirror/wine/blob/master/include/devpkey.h): หมายเลขคุณสมบัติของอุปกรณ์
- [libyal winreg-kb: อุปกรณ์จัดเก็บข้อมูล USB](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/USB-storage.md),
  [Mounted devices](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Mounted-devices.md) และ
  [Mount points](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Mount-points.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

การรวมข้อมูลผู้ใช้ใช้ตัวตนต้นทางของแต่ละไฮฟ์ ดังนั้นไฮฟ์ที่ส่งออกมาซึ่งมีชื่อโปรไฟล์เดียวกันจะยังคงแยกจากกัน `SourceKeys` เก็บพาธของไฮฟ์ผู้ใช้ที่มีส่วนให้ข้อมูลไว้ การค้นหาไฮฟ์ผู้ใช้ที่เกี่ยวข้องจะข้ามลิงก์สัญลักษณ์ (symbolic link) ซึ่งสอดคล้องกับการค้นหาไฮฟ์แบบเรียกซ้ำ การเชื่อมโยง VID/PID ต้องมี container ID ที่ตรงกันและใช้งานได้ หากไม่มี การจับคู่ด้วยหมายเลขซีเรียลต้องให้ ID ที่ไม่กำกวม ผู้สมัครที่ขัดแย้งกันจะยังคงเป็นค่าที่ไม่ทราบพร้อมบันทึกปัญหาไว้
