# โปรแกรมที่ติดตั้ง { #installed-programs }

## สิ่งที่ Windows เก็บไว้ { #what-windows-keeps }

Windows และโปรแกรมติดตั้งบันทึกซอฟต์แวร์ที่ติดตั้งไว้ในรายการหลายรายการในรีจิสทรี แต่ละรายการมีหนึ่งคีย์
ต่อหนึ่งโปรแกรม ผลิตภัณฑ์ ดิสทริบิวชัน หรือแพ็กเกจ และ Juntei ให้หนึ่งระเบียนต่อหนึ่งคีย์

| แหล่งข้อมูล | ไฮฟ์และคีย์ | สิ่งที่เก็บไว้ |
|---|---|---|
| `uninstall` | `SOFTWARE` และ `NTUSER.DAT` `…\CurrentVersion\Uninstall\*` และ `SOFTWARE\Wow6432Node\…` | สิ่งที่ Settings และ *Programs and Features* แสดง: ชื่อ เวอร์ชัน ผู้เผยแพร่ วันที่ติดตั้ง โฟลเดอร์ คำสั่งถอนการติดตั้ง ขนาด และถูกซ่อนไว้หรือไม่ |
| `msi_product` | `SOFTWARE\Classes\Installer\Products\*`, `NTUSER.DAT` `Software\Microsoft\Installer\Products\*` | ผลิตภัณฑ์ของ Windows Installer: ชื่อและเวอร์ชันของผลิตภัณฑ์ ชื่อไฟล์ของแพ็กเกจ และโฟลเดอร์ที่ใช้ติดตั้ง |
| `app_path` | `SOFTWARE` และ `NTUSER.DAT` `…\CurrentVersion\App Paths\*` | โปรแกรมที่ Windows เริ่มได้ด้วยชื่อ (`Win+R`, `start tool`): พาธของโปรแกรมและโฟลเดอร์ของโปรแกรม |
| `wsl` | `NTUSER.DAT` `…\CurrentVersion\Lxss\{GUID}` | ดิสทริบิวชันของ Windows Subsystem for Linux: ชื่อ ตำแหน่งของไฟล์หรือดิสก์ และเป็น WSL 1 หรือ 2 |
| `package` | `SOFTWARE` `…\Appx\AppxAllUserStore` | แอปแบบแพ็กเกจ (Store): ที่จัดเตรียม (provisioned) ไว้สำหรับผู้ใช้ทุกคน ของ Windows เอง ที่อยู่ในสถานะ staged ที่ถูก deprovision และของผู้ใช้แต่ละราย |

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรแกรม ผลิตภัณฑ์ หรือแพ็กเกจนี้ถูก
ติดตั้ง (หรือลงทะเบียน) บนระบบ สำหรับผู้ใช้ทุกคนหรือสำหรับผู้ใช้รายนี้ วันที่ติดตั้งคือ
สิ่งที่โปรแกรมติดตั้งเขียนไว้ ส่วนเวลาเขียนล่าสุดของคีย์คือเวลาที่รายการนั้นเปลี่ยนแปลงครั้งล่าสุด

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows\CurrentVersion\Uninstall\<name or {ProductCode}>
  Wow6432Node\Microsoft\Windows\CurrentVersion\Uninstall\<…>          32-bit programs
  Classes\Installer\Products\<packed product code>[\SourceList]
  Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Microsoft\Windows\CurrentVersion\Appx\AppxAllUserStore\{Applications,InboxApplications,Staged,Deprovisioned,<SID>,EndOfLife\<SID>}\<package>
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows\CurrentVersion\Uninstall\<…>
  Software\Microsoft\Installer\Products\<packed product code>
  Software\Microsoft\Windows\CurrentVersion\App Paths\<program.exe>
  Software\Microsoft\Windows\CurrentVersion\Lxss\{GUID}
```

ให้เก็บรวบรวม `SOFTWARE`, `NTUSER.DAT` ทุกไฟล์ และ transaction log ของไฟล์เหล่านั้น `SYSTEM` ระบุชื่อโฮสต์ และ
`ProfileList` ของ `SOFTWARE` ให้ SID ของผู้ใช้แต่ละราย

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **วันที่ติดตั้งและเวลาของคีย์** `InstallDate` เป็นวันที่ (ไม่มีเวลา) ที่โปรแกรมติดตั้งเขียนไว้ใน
  เวลาท้องถิ่น โปรแกรมติดตั้งบางตัวไม่เขียนค่านี้ และบางตัวเขียนในรูปแบบอื่น (เก็บไว้ตามที่เขียนใน
  `InstallDateRaw`) การอัปเดตอาจเขียนรายการใหม่โดยไม่เปลี่ยนวันที่ หรือเปลี่ยนวันที่
  โดยที่โปรแกรมไม่ได้เปลี่ยนแปลงมากนัก: ให้เปรียบเทียบ `InstallDate` กับ `KeyLastWrittenUTC`
- **ซอฟต์แวร์ที่ถอนการติดตั้งแล้ว** โปรแกรมที่ถูกลบอย่างเรียบร้อยจะสูญเสียคีย์ไป ให้ใช้ `parse-registry read
  --recover-deleted` สำหรับคีย์ `Uninstall` ที่ถูกลบ และใช้ Amcache, Prefetch และ `$MFT` เพื่อหาร่องรอย
  ที่หลงเหลืออยู่
- **การลงทะเบียนต่อผู้ใช้** รายการใน `NTUSER.DAT` ระบุซอฟต์แวร์ที่ลงทะเบียนไว้สำหรับโปรไฟล์หนึ่ง
  ขอบเขตนี้ไม่ได้ยืนยันว่าใครเป็นผู้เรียกใช้โปรแกรมติดตั้ง หรือมีการใช้สิทธิ์ผู้ดูแลระบบหรือไม่
- **รายการที่ถูกซ่อน** `SystemComponent` ซ่อนรายการจาก *Programs and Features* และรายการ
  ที่ไม่มี `DisplayName` (อัปเดต คอมโพเนนต์) ก็ไม่แสดงเช่นกัน แต่ทั้งหมดจะถูกรายงาน
- **Product code** คีย์ `Uninstall` ที่มีรูปแบบเป็น GUID สามารถจับคู่กับคีย์
  `msi_product` แบบ packed ผ่าน `ProductCode` ได้ ชื่อที่มีรูปแบบเป็น GUID เพียงอย่างเดียวไม่ได้พิสูจน์ว่า
  Windows Installer เป็นผู้สร้างรายการนั้น
- **App Paths** เป็นการลงทะเบียนที่ใช้ค้นหาตำแหน่งโปรแกรม ไม่ใช่ระเบียนการเรียกใช้งาน
  ค่า `Path` อาจมีไดเรกทอรีค้นหาหลายรายการ และไม่จำเป็นต้องเป็น
  ไดเรกทอรีที่ติดตั้ง
- **ผู้เผยแพร่** เป็นเมทาดาทาในรีจิสทรีที่โปรแกรมติดตั้งระบุ ไม่ใช่ข้อมูลลายเซ็นที่
  ผ่านการตรวจสอบแล้ว
- **สถาปัตยกรรม** `Wow6432Node` เก็บโปรแกรม 32 บิตบน Windows 64 บิต (`Architecture` `x86`,
  `Wow64` `True`) รายการอื่นเป็นสถาปัตยกรรมของระบบเอง (ว่างเปล่า) สถาปัตยกรรมของแพ็กเกจ
  เป็นของแพ็กเกจเอง (`x64`, `neutral`, …)
- **WSL** `Version` คือเวอร์ชัน WSL จาก `Flags` ของดิสทริบิวชัน (`0x8`: ทำงานใน
  virtual machine ของ WSL 2) ค่า `Version` ของดิสทริบิวชันเอง (ใน `Details`) คือเวอร์ชันของ
  ระบบไฟล์ ซึ่งเป็น 2 บนดิสทริบิวชัน WSL 1 ด้วย `BasePath` เก็บ `ext4.vhdx` (WSL 2) หรือ `rootfs`
  (WSL 1): ให้เก็บรวบรวมไว้เพื่อตรวจดูข้างใน
- **แพ็กเกจ** รายการ `<SID>` คือแพ็กเกจที่ลงทะเบียนไว้สำหรับผู้ใช้แต่ละราย ส่วนใหญ่เป็นของ Windows
  เอง แพ็กเกจ `Deprovisioned` ถูกลบออกสำหรับผู้ใช้ใหม่ ส่วน `EndOfLife\<SID>` ถูกลบสำหรับผู้ใช้รายนั้น
- **ค่าที่เสียหาย** ตัวเลขที่ผิดรูปแบบ ชนิดของค่าที่ไม่ถูกต้อง และ UTF-16 ที่ไม่ถูกต้องจะถูกรายงานใน
  `Problems` และเก็บไว้ใน `DataRaw` รวมถึงสตริง hex ว่างสำหรับค่าที่มีอยู่แต่มี
  ศูนย์ไบต์ สตริงว่างที่ไม่มีปัญหาในฟิลด์ตัวเลขที่ไม่บังคับจะถือว่าไม่มีค่า เวอร์ชันของ
  แพ็กเกจต้องมีส่วนประกอบเลขฐานสิบสี่ส่วน แต่ละส่วนอยู่ในช่วง unsigned 16 บิต
  คำเตือนระดับไฮฟ์จะยังคงอยู่ใน stderr เมื่อ `-t` สตรีมระเบียนไปยัง stdout

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry installed-programs <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry installed-programs -d evidence/ -o installed.csv
juntei parse-registry installed-programs -d evidence/ -t jsonl \
  | jq 'select(.Scope == "user" and .Source != "package")'
```

ส่วนสรุปแสดงโปรแกรมที่ติดตั้งล่าสุด (ตาม `InstallDate`) สิ่งที่ผู้ใช้ติดตั้งไว้สำหรับ
ตนเอง และดิสทริบิวชัน WSL

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | โฮสต์ ไฟล์ไฮฟ์ และ `SOFTWARE` หรือ `NTUSER.DAT` |
| `User`, `UserSID` | ผู้ใช้ของไฮฟ์ผู้ใช้ (โฟลเดอร์โปรไฟล์และ SID จาก `ProfileList`) หรือผู้ใช้ที่แพ็กเกจถูกแสดงไว้ |
| `Source`, `Scope` | รายการ (`uninstall`, `msi_product`, `app_path`, `wsl`, `package`) และ `machine` หรือ `user` |
| `Name`, `Version`, `Publisher` | `DisplayName`, `DisplayVersion`, `Publisher`; `ProductName` และ `Version` ของผลิตภัณฑ์; ชื่อไฟล์โปรแกรมของ App Path; ชื่อและเวอร์ชัน WSL ของดิสทริบิวชัน; ชื่อและเวอร์ชันของแพ็กเกจ |
| `InstallDate`, `InstallDateRaw` | `InstallDate` ในรูปแบบวันที่เมื่อเขียนเป็น `YYYYMMDD` มิฉะนั้นจะเป็นไปตามที่เขียนไว้ |
| `KeyLastWrittenUTC` | เวลาที่คีย์ของรายการเปลี่ยนแปลงครั้งล่าสุด |
| `InstallLocation`, `InstallSource` | `InstallLocation` (`Path` ของ App Path, `BasePath` ของดิสทริบิวชัน, `Path` ของแพ็กเกจ); `InstallSource` (โฟลเดอร์ต้นทางที่ผลิตภัณฑ์ใช้ครั้งล่าสุด) |
| `Program` | โปรแกรมของ App Path |
| `UninstallCommand`, `QuietUninstallCommand` | `UninstallString`, `QuietUninstallString` |
| `SizeKB`, `SystemComponent`, `WindowsInstaller` | `EstimatedSize` เป็น KiB; ถูกซ่อนไว้หรือไม่; ติดตั้งโดย Windows Installer หรือไม่ |
| `ProductCode` | product code ของ Windows Installer (`{GUID}`) |
| `Architecture`, `WOW64` | `x86` ใต้ `Wow6432Node` หรือสถาปัตยกรรมของแพ็กเกจ; คีย์อยู่ใต้ `Wow6432Node` หรือไม่ |
| `Details` | ข้อมูลเพิ่มเติม: `DisplayIcon`, `ModifyPath`, `ParentKeyName`, `ReleaseType`, `Inno Setup: User` (บัญชีที่เรียกใช้โปรแกรมติดตั้ง Inno Setup); `PackageCode`, `PackageName` ของผลิตภัณฑ์; `Flags`, `State`, `DefaultUid`, `Version`, `PackageFamilyName`, `Default` ของดิสทริบิวชัน; `List`, `ResourceId`, `PublisherId` ของแพ็กเกจ |
| `KeyPath` | คีย์ของรายการ |
| `DataRaw` | ไบต์ (hex) ของค่าที่ไม่ตรงกับสิ่งที่ชื่อระบุ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่อ่านไม่ได้หรือไม่ตรงกับสิ่งที่ชื่อระบุ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงตาม `InstallDate` และ `KeyLastWrittenUTC` ในช่วงเวลาของเหตุการณ์ และพิจารณาเครื่องมือ
   การเข้าถึงระยะไกล ไคลเอนต์ tunnelling และการถ่ายโอนไฟล์ โปรแกรมบีบอัดไฟล์ runtime ของภาษาสคริปต์ และเครื่องมือด้านความปลอดภัย
2. ตรวจทานระเบียนที่มี `Scope` เป็น `user`: ซอฟต์แวร์ที่ติดตั้งโดยไม่ใช้สิทธิ์ผู้ดูแลระบบ
3. ตรวจสอบ `InstallLocation`, `InstallSource` และ `UninstallCommand` ว่าชี้ไปยังโฟลเดอร์ที่ผู้ใช้เขียนได้หรือไม่
   (`Downloads`, `AppData`, `Temp`, `ProgramData`)
4. มองหาดิสทริบิวชัน WSL และเก็บรวบรวม `BasePath` ของดิสทริบิวชันเหล่านั้น
5. เชื่อมโยงกับ Amcache และ Prefetch (โปรแกรมติดตั้งและโปรแกรมที่ทำงาน), `$MFT` (การสร้าง
   โฟลเดอร์ติดตั้ง) และ Application event log (เหตุการณ์ MsiInstaller)

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: คีย์รีจิสทรี Uninstall](https://learn.microsoft.com/en-us/windows/win32/msi/uninstall-registry-key)
- [Microsoft: การลงทะเบียนแอป (App Paths)](https://learn.microsoft.com/en-us/windows/win32/shell/app-registration)
- [Microsoft: ภาพรวมของ package identity](https://learn.microsoft.com/en-us/windows/apps/desktop/modernize/package-identity-overview)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
