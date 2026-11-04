# MuiCache { #muicache }

## MuiCache คืออะไร { #what-muicache-is }

เมื่อผู้ใช้เริ่มโปรแกรม Windows จะอ่าน version resource ของโปรแกรมเพื่อหาชื่อที่
shell ควรแสดง (`FileDescription` เช่น "Windows Command Processor" สำหรับ
`cmd.exe`) และแคชไว้ต่อผู้ใช้ใน `MuiCache` Windows 8 ขึ้นไปยังแคชชื่อบริษัท
(`CompanyName`) ด้วย แคชนี้ใช้พาธเต็มของโปรแกรมเป็นคีย์:

- **Windows 8 ขึ้นไป:** สองค่าต่อหนึ่งโปรแกรม คือ `<path>.FriendlyAppName` และ
  `<path>.AppCompany`
- **Windows Vista และ 7:** หนึ่งค่าต่อหนึ่งโปรแกรม คือ `<path>` ซึ่งเก็บชื่อที่แสดง (friendly name)
- **Windows XP:** รูปแบบเดียวกัน แต่อยู่ใน `NTUSER.DAT`

MuiCache สามารถให้เบาะแสการเรียกใช้โปรแกรมใน shell session ของผู้ใช้ รวมถึงโปรแกรม
บนไดรฟ์แบบถอดได้ แชร์บนเครือข่าย และโฟลเดอร์ที่ไม่มีอยู่แล้ว พร้อมชื่อและบริษัท
ที่ตัวไฟล์ระบุไว้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรไฟล์ shell นี้ได้แคชพาธ
คำอธิบาย และบริษัทของไฟล์ไว้ ให้เชื่อมโยงกับอาร์ติแฟกต์การเรียกใช้งานอื่น ๆ โปรไฟล์ไม่ได้ระบุ
บัญชีที่โพรเซสทำงานในฐานะนั้น MuiCache ไม่มีเวลาต่อโปรแกรมและไม่มีจำนวนครั้งที่เรียกใช้

## ตำแหน่ง { #location }

```text
C:\Users\<user>\AppData\Local\Microsoft\Windows\UsrClass.dat     Windows Vista and later
Local Settings\Software\Microsoft\Windows\Shell\MuiCache

C:\Documents and Settings\<user>\NTUSER.DAT                         Windows XP
Software\Microsoft\Windows\ShellNoRoam\MUICache
```

`LangID` (ภาษาของสตริงที่แคชไว้) ไม่ใช่โปรแกรมและจะถูกละไว้ ค่าที่มีชื่อเป็น
`@<file>,-<number>` (`@shell32.dll,-22019`) คือสตริงจาก resource ที่ shell แคชไว้สำหรับเมนูและชนิดไฟล์
ของตัวเอง ไม่ใช่โปรแกรม จึงถูกละไว้และนับรวมในส่วนสรุป บน Windows Vista
ขึ้นไป `NTUSER.DAT` ไม่มี MuiCache

ให้เก็บรวบรวม `UsrClass.dat` (และ `NTUSER.DAT`) พร้อม transaction log ของทุกโปรไฟล์ และไฮฟ์
`SYSTEM` และ `SOFTWARE` ของระบบเพื่อระบุชื่อโฮสต์และชื่อผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **เจ้าของโปรไฟล์ไม่จำเป็นต้องเป็นบัญชีของโพรเซส** โปรแกรมที่เริ่มด้วยบัญชี
  อื่นสามารถแสดงหน้าต่างใน session ของผู้ใช้รายนี้และทิ้งเมทาดาทาไว้ในแคชของผู้ใช้รายนี้ได้
  `User` และ `UserSid` ระบุเจ้าของไฮฟ์ ไม่ใช่ security context ของโพรเซส
- **ไม่มีเวลาต่อโปรแกรม** `KeyLastWrittenUTC` คือเวลาเขียนล่าสุดของคีย์: การเปลี่ยนแปลงครั้งล่าสุดของ
  โปรแกรมใดโปรแกรมหนึ่งในคีย์ ไม่ใช่เวลาที่โปรแกรมใดโปรแกรมหนึ่งทำงาน Prefetch, BAM, UserAssist และ event log ให้
  เวลาได้
- **ไม่ใช่เฉพาะโปรแกรมที่ผู้ใช้เริ่มด้วยตนเอง** DLL หรือโปรแกรมที่ shell โหลดเอง (สำหรับ
  `Open with`, shell extension หรือ child ของโปรแกรมติดตั้ง) อาจปรากฏได้ เช่น `explorerframe.dll` หรือ
  `shell32.dll`
- **ชื่อมาจากตัวไฟล์** `FriendlyAppName` และ `AppCompany` คือสิ่งที่ version resource ของไฟล์
  ระบุไว้ขณะที่ทำงาน มัลแวร์สามารถอ้างคำอธิบายและบริษัทใดก็ได้ เครื่องมือที่ถูกเปลี่ยนชื่อ
  จะยังคงคำอธิบายเดิมไว้ ซึ่งอาจเปิดเผยตัวตนได้ (ไฟล์ชื่อ `svchost.exe` ที่ชื่อที่แสดง
  เป็นเครื่องมือเข้าถึงระยะไกล)
- **หนึ่งรายการต่อหนึ่งพาธ** การเรียกใช้โปรแกรมเดิมซ้ำจะไม่เพิ่มรายการ แต่สำเนาที่อยู่ในพาธอื่น
  จะเพิ่มรายการ รายการของโปรแกรมอาจยังคงอยู่หลังจากไฟล์ถูกลบไปแล้ว
- **ทั้งสองรูปแบบสำหรับพาธเดียว** เมื่อพาธหนึ่งมีชื่อที่แสดงในทั้งสองรูปแบบ (โปรไฟล์ที่
  อัปเกรดมา) แต่ละรูปแบบจะได้ระเบียนของตัวเอง ทำให้ไม่มีค่าใดสูญหาย
- **ค่าที่เสียหาย** ค่าที่ไม่ใช่สตริง ไม่ใช่ UTF-16 ที่ถูกต้อง หรืออ่านได้ไม่ครบ จะเก็บ
  ไบต์ไว้ (`FriendlyAppNameRaw`, `AppCompanyRaw`) และถูกรายงานใน `Problems`
  UTF-16 ที่ไม่ถูกต้องอาจทำให้เกิดอักขระแทนที่ (replacement character) ในข้อความที่แสดงด้วย ให้ใช้คอลัมน์ raw
  สำหรับไบต์ต้นฉบับ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry muicache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find UsrClass.dat and NTUSER.DAT hives
  -f, --file <FILE>      Parse one UsrClass.dat or NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry muicache -d evidence/ -o muicache.csv
juntei parse-registry muicache -f UsrClass.dat -t jsonl \
  | jq 'select(.Path | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

ส่วนสรุปแสดงโปรแกรมที่อยู่นอกโฟลเดอร์ Windows และ Program Files (โฟลเดอร์ของผู้ใช้
ไดรฟ์อื่น แชร์บนเครือข่าย) พร้อมผู้ใช้ และจำนวนสตริง resource ที่ถูกละไว้
โฟลเดอร์ Program Files ที่มีชื่ออื่น (บน Windows ภาษาอื่น) ก็ถูกแสดงด้วย
พาธสัมพัทธ์และพาธที่หลุดออกจากโฟลเดอร์ตัวแปรสภาพแวดล้อมจะยังคงถูกแสดง พาธของไดรฟ์
ทั่วไปจะถูกตรวจสอบหลังจากแปลงส่วนประกอบ `.` และ `..` แล้ว โดยไม่เข้าถึงระบบไฟล์ของ
ผู้ตรวจสอบ

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ ชื่อโฟลเดอร์โปรไฟล์ SID ของผู้ใช้ (คีย์รากของ `UsrClass.dat` ระบุไว้ มิฉะนั้นใช้ `ProfileList` ของระบบ) และไฮฟ์ |
| `HiveType` | `UsrClass.dat` หรือ `NTUSER.DAT` |
| `Path` | พาธของโปรแกรม: ชื่อของค่าโดยไม่มี `.FriendlyAppName` หรือ `.AppCompany` |
| `FriendlyAppName` | คำอธิบายของโปรแกรม จาก `<path>.FriendlyAppName` หรือ `<path>` |
| `AppCompany` | บริษัทของโปรแกรม จาก `<path>.AppCompany` (Windows 8 ขึ้นไป) |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์ MuiCache และเวลาเขียนล่าสุด: การเปลี่ยนแปลงครั้งล่าสุดของโปรแกรมใดโปรแกรมหนึ่งในคีย์ |
| `FriendlyAppNameRaw`, `AppCompanyRaw` | ไบต์ของค่า (hex) เมื่อไม่ใช่ข้อความที่ถอดรหัสได้อย่างสมบูรณ์ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี: `replayed` เมื่อ log ได้เขียนค่าใดค่าหนึ่งของโปรแกรม |
| `Problems` | ค่าที่ไม่ใช่สตริง ไม่ใช่ UTF-16 ที่ถูกต้อง หรืออ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. พิจารณาโปรแกรมในตำแหน่งที่ผู้ใช้เขียนได้ (`\Users\`, `Downloads`, `Temp`, `ProgramData`) บน
   ไดรฟ์อื่นและบนแชร์เครือข่าย รวมถึงเครื่องมือดูแลระบบ การเข้าถึงระยะไกล และการบีบอัดไฟล์
2. เปรียบเทียบชื่อไฟล์กับชื่อที่แสดงและบริษัท: ความไม่ตรงกันอาจเปิดเผยเครื่องมือที่ถูกเปลี่ยนชื่อ
   หรือไฟล์ที่ปลอมตัว
3. เชื่อมโยงกับ Prefetch, BAM และ UserAssist (เวลาที่ทำงาน), Amcache และ ShimCache (แฮชและ
   การมีอยู่ของไฟล์) และ `$MFT` (ไฟล์ยังอยู่หรือไม่)
4. ใช้ `parse-registry read --recover-deleted` กับ `UsrClass.dat` สำหรับค่า MuiCache ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: VERSIONINFO resource](https://learn.microsoft.com/en-us/windows/win32/menurc/versioninfo-resource)
- [Cyber Triage: การทดลองกับ MuiCache, GUI session และข้อจำกัดด้านบัญชีของโพรเซส](https://www.cybertriage.com/blog/muicache-2026-guide/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
