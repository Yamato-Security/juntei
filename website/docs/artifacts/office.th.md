# Microsoft Office (ไฟล์ล่าสุด, Trusted Documents, การตั้งค่ามาโคร) { #microsoft-office-recent-files-trusted-documents-macro-settings }

## สิ่งที่ Office บันทึกไว้ { #what-office-records }

Microsoft Office จัดเก็บข้อมูลต่อผู้ใช้ไว้ใน `NTUSER.DAT` แยกตามเวอร์ชันและแอปพลิเคชัน ได้แก่

- **ไฟล์และตำแหน่งล่าสุด:** `File MRU` คือเอกสารที่แอปพลิเคชันเปิด และ `Place MRU`
  คือโฟลเดอร์ที่เอกสารเหล่านั้นถูกเปิดมา แต่ละรายการมีเวลาที่เปิดครั้งล่าสุด Microsoft 365
  ยังเก็บรายการแยกต่อบัญชีที่ลงชื่อเข้าใช้ (`User MRU\<account>`) ด้วย
- **Trusted Documents:** เมื่อผู้ใช้คลิก *Enable Editing* หรือ *Enable Content* บนเอกสาร
  Office จะจดจำเอกสารนั้นไว้ใน `TrustRecords` ได้แก่ พาธของเอกสาร เวลาที่ได้รับความเชื่อถือ และ
  มาโครของเอกสารถูกเปิดใช้งานหรือไม่
- **การตั้งค่ามาโคร:** `VBAWarnings` (มาโครใดบ้างที่ทำงาน), `BlockContentExecutionFromInternet`
  (มาโครในไฟล์จากอินเทอร์เน็ตถูกบล็อกหรือไม่), `AccessVBOM` (โปรแกรมสามารถ
  แก้ไขโปรเจกต์ VBA ได้หรือไม่) และ **Trusted Locations** ซึ่งเป็นโฟลเดอร์ที่มาโครของเอกสารภายในจะทำงานโดยไม่
  ถาม Group Policy สามารถตั้งค่าเดียวกันนี้ได้ภายใต้ `Software\Policies`

เอกสารที่เป็นอันตรายโดยปกติต้องให้ผู้ใช้เปิดใช้งานมาโคร: trust record ที่มาโคร
ถูกเปิดใช้งานจะเชื่อมโยงเอกสาร ผู้ใช้ และเวลาเข้าด้วยกัน

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้เปิดเอกสารนี้ในแอปพลิเคชันนี้
(และเปิดครั้งล่าสุด ณ เวลานี้) ผู้ใช้เปิดใช้งานการแก้ไขหรือมาโครของเอกสารนี้ ณ
เวลานี้ และ Office ถูกกำหนดค่าให้เรียกใช้มาโครในลักษณะนี้

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Office\<version>\<application>\File MRU                   Item 1, Item 2, …
Software\Microsoft\Office\<version>\<application>\Place MRU
Software\Microsoft\Office\<version>\<application>\User MRU\<account>\File MRU
Software\Microsoft\Office\<version>\<application>\Security\Trusted Documents\TrustRecords
Software\Microsoft\Office\<version>\<application>\Security                   VBAWarnings, …
Software\Microsoft\Office\<version>\<application>\Security\Trusted Locations\LocationN
Software\Policies\Microsoft\Office\<version>\<application>\Security          by policy
```

เวอร์ชัน: `16.0` คือ Office 2016 และรุ่นใหม่กว่า (2019, 2021, 2024, Microsoft 365), `15.0` คือ Office 2013,
`14.0` คือ Office 2010, `12.0` คือ Office 2007

แต่ละรายการมีลักษณะเช่น `[F00000000][T01D86AF6D3C287B0][O00000000]*C:\Users\alice\report.docx` โดย `T`
คือเวลาที่เปิดครั้งล่าสุด (FILETIME ในรูปเลขฐานสิบหก) และพาธตามหลัง `*` trust record มีขนาด 24
ไบต์ ได้แก่ FILETIME ที่เอกสารได้รับความเชื่อถือ ตามด้วยแฟล็กใน 4 ไบต์สุดท้าย ซึ่งเป็น `0x7FFFFFFF`
เมื่อมาโครถูกเปิดใช้งาน และ `0x00000001` เมื่อเปิดใช้งานเฉพาะการแก้ไข

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **เวลา** `LastOpenedUTC` คือเวลาของแต่ละรายการเอง `TrustedAtUTC` คือเวลาที่ผู้ใช้ให้ความเชื่อถือ
  เอกสาร ซึ่งก็คือช่วงเวลาที่คลิก *Enable Editing* หรือ *Enable Content* ซึ่งอาจอยู่ก่อน
  การเปิดครั้งล่าสุด
- **มาโครถูกเปิดใช้งาน ไม่ใช่ถูกเรียกใช้** `MacrosEnabled` บอกว่าผู้ใช้เปิดใช้งานมาโครของเอกสาร แต่
  ไม่ได้พิสูจน์ว่ามาโครทำงานหรือทำอะไร ให้เชื่อมโยงกับการสร้างโพรเซส Prefetch บันทึก PowerShell
  และตัวเอกสารเอง
- **พาธตามที่ Office จัดเก็บ** trust record ระบุเอกสารด้วยพาธที่อาจใช้
  `%USERPROFILE%` เครื่องหมายทับไปข้างหน้า URL แบบ `file:///` และการเข้ารหัส URL (`%20`) รายการ MRU ของ
  เอกสารบนคลาวด์จะเป็น URL
- **ค่าเริ่มต้นและนโยบายที่มีผลจริง** Office สร้าง trusted location เริ่มต้น (`Templates`,
  `XLSTART`, `STARTUP`, …) ค่า `Date` ของ location เป็นเบาะแสที่มีประโยชน์ แต่ไม่ใช่หลักฐานว่าใครเป็นผู้สร้าง:
  ผู้ดูแลระบบ สคริปต์ นโยบาย และการตั้งค่ารีจิสทรีที่ถูกคัดลอกมาก็สามารถเพิ่มค่านี้ได้ การไม่มี
  `VBAWarnings` เพียงอย่างเดียวไม่ได้ยืนยันนโยบายมาโครที่มีผลจริง เวอร์ชันของ Office การทำเครื่องหมายไฟล์
  จากอินเทอร์เน็ต และการตั้งค่าการจัดการอื่น ๆ ก็มีผลเช่นกัน
  [Microsoft อธิบายการบล็อกมาโครจากอินเทอร์เน็ต](https://learn.microsoft.com/en-us/DeployOffice/security/internet-macros-blocked)
- **เอกสารที่ถูกลบ** รายการเหล่านี้คงอยู่นานกว่าตัวเอกสาร ไฟล์ที่ไม่มีอยู่แล้วอาจยังแสดง
  ที่นี่ได้ รายการมีความยาวสั้นและจะถูกล้างโดย *Clear unpinned items* ของ Office และเครื่องมือล้างข้อมูล

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry office <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry office -d evidence/ -o office.csv
juntei parse-registry office -f NTUSER.DAT -t jsonl | jq 'select(.MacrosEnabled == true)'
```

ข้อมูลสรุปจะแสดงเอกสารที่มาโครถูกเปิดใช้งาน การตั้งค่าที่ทำให้มาโครทำงานได้
อย่างอิสระมากขึ้น (`VBAWarnings` 1, `BlockContentExecutionFromInternet` 0, `AccessVBOM` 1,
`AllowNetworkLocations` 1) trusted location ที่ผู้ใช้เพิ่มหรือนโยบายกำหนด และเอกสารที่
เปิดล่าสุด

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ เจ้าของโปรไฟล์ (จากพาธหรือ shell folder) SID จาก `ProfileList` ของระบบ และไฮฟ์ |
| `OfficeVersion`, `App` | คีย์ของเวอร์ชัน (`16.0`) และแอปพลิเคชัน (`Word`, `Excel`, …) |
| `List` | `File MRU`, `Place MRU`, `TrustRecords`, `Security` หรือ `Trusted Locations` |
| `Account` | บัญชี Microsoft 365 ของรายการ `User MRU` |
| `Policy` | การตั้งค่าหรือ location ถูกกำหนดโดยนโยบายหรือไม่ |
| `MRU-Position`, `IsMostRecent` | ตำแหน่งของรายการในลิสต์ (`Item 1` คือ 0 ซึ่งเป็นรายการล่าสุด) |
| `Path` | เอกสารหรือโฟลเดอร์ พาธของ trusted document ตามที่จัดเก็บ หรือ `Path` ของ trusted location |
| `LastOpenedUTC` | เวลาที่รายการถูกเปิดครั้งล่าสุด |
| `TrustedAtUTC`, `MacrosEnabled`, `TrustFlags` | เวลาที่เอกสารได้รับความเชื่อถือ มาโครถูกเปิดใช้งานหรือไม่ และแฟล็กของเรคคอร์ด แฟล็กที่ไม่รู้จักจะทำให้ `MacrosEnabled` ว่างและสร้างข้อความวินิจฉัย |
| `Setting`, `Value` | การตั้งค่ามาโครและค่าพร้อมความหมาย (`1 (all macros enabled)`) หรือคีย์ของ trusted location (`Location3`) และค่าอื่น ๆ ของคีย์นั้น (`AllowSubFolders=1; Date=…; Description=…`) |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์และเวลาเขียนล่าสุด |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | รายการที่ไม่มี `*` หรือมีเวลาที่ไม่ใช่เวลาที่ถูกต้อง trust record ที่ไม่ใช่ 24 ไบต์ location ที่ไม่มี `Path` และชนิดที่ไม่ถูกต้อง |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เริ่มจาก `MacrosEnabled` ที่เป็น true: เอกสารใด เมื่อใด และโดยใคร ค้นหาไฟล์ (`$MFT`,
   USN Journal, `Zone.Identifier`) แหล่งที่มา (ประวัติเบราว์เซอร์และอีเมล) และสิ่งที่ทำงานต่อจากนั้น
2. ตรวจสอบการตั้งค่ามาโครและ trusted location: การเปิดใช้งานมาโครทั้งหมด การไม่บล็อกมาโครจากอินเทอร์เน็ต
   การเข้าถึงโปรเจกต์ VBA หรือ location ที่ผู้ใช้เพิ่มเอง (ใน `Downloads`, `Temp` หรือแชร์บนเครือข่าย)
   ล้วนลดความปลอดภัยของค่าเริ่มต้น
3. นำ `LastOpenedUTC` ไปไว้บนไทม์ไลน์ร่วมกับไฟล์ LNK, Jump Lists และ RecentDocs สำหรับ
   พาธของเอกสาร รวมถึงตำแหน่งแบบถอดได้และบนเครือข่าย

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: เปิดหรือปิดใช้งานมาโครในไฟล์ Microsoft 365](https://support.microsoft.com/en-us/office/enable-or-disable-macros-in-microsoft-365-files-12b036fd-d140-4e74-b45e-16fed1a7e5c6)
- [Microsoft: Trusted documents](https://support.microsoft.com/en-us/office/trusted-documents-cf872bd8-47ec-4c02-baa5-1fdba1a11b53)
- [Microsoft: เพิ่ม ลบ หรือเปลี่ยน trusted location](https://support.microsoft.com/en-us/office/add-remove-or-change-a-trusted-location-in-microsoft-office-7ee1cdc2-483e-4cbb-bcb3-4e7c67147fb4)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
