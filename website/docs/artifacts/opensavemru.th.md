# กล่องโต้ตอบเปิดและบันทึกเป็น (OpenSaveMRU) { #open-and-save-as-dialogs-opensavemru }

## ประวัติของกล่องโต้ตอบคืออะไร { #what-the-dialog-history-is }

เมื่อโปรแกรมแสดงกล่องโต้ตอบ Open หรือ Save As มาตรฐาน Windows จะบันทึกข้อมูลต่อไปนี้ไว้ใน
`NTUSER.DAT` ของผู้ใช้

- **ไฟล์ใดถูกเปิดหรือบันทึก:** `OpenSavePidlMRU` มีคีย์ย่อยหนึ่งคีย์ต่อหนึ่งนามสกุล (`docx`,
  `ps1`, …) และ `*` สำหรับทุกนามสกุล ซึ่งแต่ละคีย์เป็นรายการที่ใช้ล่าสุด (most-recently-used)
- **โปรแกรมใดใช้กล่องโต้ตอบ และโฟลเดอร์ที่กล่องโต้ตอบอยู่:** `LastVisitedPidlMRU` (และ
  `LastVisitedPidlMRULegacy` สำหรับโปรแกรมที่ใช้กล่องโต้ตอบแบบเก่า)
- **โปรแกรมที่แสดงกล่องโต้ตอบ:** `CIDSizeMRU` (ขนาดของกล่องโต้ตอบต่อโปรแกรม) และ
  `FirstFolder` (พาธของโปรแกรมและโฟลเดอร์)
- บน Windows 2000 และ XP มี `OpenSaveMRU` (พาธต่อนามสกุล) และ `LastVisitedMRU` (โปรแกรมและ
  โฟลเดอร์ในรูปพาธ)

ตั้งแต่ Windows Vista ไฟล์และโฟลเดอร์จะถูกเก็บเป็นรายการ shell item ซึ่งเป็นโครงสร้างเดียวกับเป้าหมายของ
ทางลัด: แต่ละ item เป็นส่วนหนึ่งของพาธ (This PC, ไดรฟ์, โฟลเดอร์, ไฟล์) และ
item ของไฟล์หรือโฟลเดอร์สามารถเก็บเวลาและ NTFS file reference ของตนเองได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรแกรมใช้กล่องโต้ตอบ Open หรือ Save As
เพื่อเปิดหรือบันทึกไฟล์นี้ หรือในโฟลเดอร์นี้ ในเซสชันของผู้ใช้นี้ มีเพียงรายการล่าสุด
ของแต่ละลิสต์เท่านั้นที่มีเวลากำกับ ซึ่งก็คือเวลาเขียนล่าสุดของคีย์

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\ComDlg32\
  OpenSavePidlMRU\<extension>   shell item lists, MRUListEx order (Vista and later)
  OpenSavePidlMRU\*             every extension
  LastVisitedPidlMRU            program name, then a shell item list
  LastVisitedPidlMRULegacy      the same
  CIDSizeMRU                    program name
  FirstFolder                   program path, then a folder path
  OpenSaveMRU\<extension>       paths, MRUList order (2000 and XP)
  LastVisitedMRU                program name, then a folder path (2000 and XP)
```

ให้เก็บรวบรวม `NTUSER.DAT` พร้อมบันทึกธุรกรรมของทุกโปรไฟล์

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **หนึ่งเวลาต่อหนึ่งลิสต์** เวลาเขียนล่าสุดของคีย์จะเปลี่ยนเมื่อรายการใดรายการหนึ่งถูกย้ายไป
  ด้านหน้าของลิสต์ ดังนั้นเวลานี้ระบุเวลาได้เฉพาะรายการที่ตำแหน่ง 0 (`IsMostRecent`, `LastUsedUTC`)
  รายการอื่น ๆ เรียงลำดับจากใหม่สุดไปเก่าสุดตามลิสต์ แต่ไม่มีเวลากำกับ
- **ไฟล์เดียวกันอยู่ในสองลิสต์** ไฟล์จะอยู่ทั้งในลิสต์ของนามสกุลและใน `*` อย่านับเป็น
  การใช้งานสองครั้ง `*` ให้ลำดับข้ามนามสกุลทั้งหมด
- **เปิดหรือบันทึก** ลิสต์ไม่ได้ระบุว่าเป็นอย่างใด ให้เชื่อมโยงกับเรคคอร์ด `$MFT` และ USN
  Journal ของไฟล์ ไฟล์ LNK และ Jump Lists
- **เวลาของ item เป็นของไฟล์ ไม่ใช่ของกล่องโต้ตอบ** `ItemModifiedUTC`, `ItemCreatedUTC` และ
  `ItemAccessedUTC` คือเวลาของไฟล์ ณ ตอนที่ item ถูกเขียน ละเอียดถึง 2 วินาที (FAT) item ของ
  known folder (`CLSID_ThisPCDocumentsRegFolder`, `Downloads`, …) มักไม่มีเวลา timestamp แบบ FAT เหล่านี้
  ไม่มี offset ของเขตเวลา คำต่อท้ายคอลัมน์ `UTC` เป็นเพียงรูปแบบการแสดงผล ไม่ใช่
  หลักฐานว่านาฬิกาต้นทางเป็น UTC ให้เชื่อมโยงกับเขตเวลาของระบบต้นทางและ
  timestamp อื่น ๆ ก่อนเปรียบเทียบกับ FILETIME ของรีจิสทรี ดู
  [แนวทางเรื่องเวลาของไฟล์](https://learn.microsoft.com/en-us/windows/win32/sysinfo/file-times) ของ Microsoft
- **พาธเป็น shell path** พาธเริ่มต้นที่เดสก์ท็อป: `This PC\C:\Users\…`, known folder,
  ไลบรารี (`UsersLibraries\Documents\…`) หรือตำแหน่งบนเครือข่าย พาธที่เริ่มต้นด้วยโฟลเดอร์
  หรือไฟล์จะอยู่บนเดสก์ท็อป พาธที่ว่างคือตัวเดสก์ท็อปเอง
- **item ที่ถอดรหัสไม่ได้** item ที่มีชนิดซึ่งพาร์เซอร์ shell item ไม่รู้จักจะแสดงด้วย
  คลาส (`[class 0x00]`) ชนิดเป็น `unknown` และไบต์ของลิสต์จะอยู่ใน `DataRaw`
- **โปรแกรมถูกเก็บเป็นชื่อ** `LastVisitedPidlMRU` และ `CIDSizeMRU` เก็บชื่อไฟล์
  (`WINWORD.EXE`) ไม่ใช่พาธ ส่วน `FirstFolder` เก็บพาธ
- **ลิสต์มีความยาวสั้นและถูกล้างได้** แต่ละลิสต์เก็บรายการได้จำนวนจำกัด และเครื่องมือล้างข้อมูล
  จะลบคีย์เหล่านี้ ใช้ `parse-registry read --recover-deleted` สำหรับค่าที่ถูกลบ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry open-save-mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs
      --code-page <CP>   Code page of shell item names that aren't Unicode (default 1252)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry open-save-mru -d evidence/ -o opensavemru.csv
juntei parse-registry open-save-mru -f NTUSER.DAT -t jsonl \
  | jq 'select(.List == "OpenSavePidlMRU" and .Extension != "*")'
```

ข้อมูลสรุปจะแสดงรายการต่อลิสต์ โปรแกรมที่ใช้กล่องโต้ตอบ และไฟล์ล่าสุดของแต่ละไฮฟ์
(รายการแรกของ `*`)

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ เจ้าของโปรไฟล์ (จากพาธหรือ shell folder) SID จาก `ProfileList` ของระบบ และไฮฟ์ |
| `List` | `OpenSavePidlMRU`, `LastVisitedPidlMRU`, `LastVisitedPidlMRULegacy`, `CIDSizeMRU`, `FirstFolder`, `OpenSaveMRU` หรือ `LastVisitedMRU` |
| `Extension` | คีย์ย่อยของนามสกุล (`docx`, `*`) ว่างสำหรับลิสต์อื่น |
| `MRU-Position`, `IsMostRecent` | ตำแหน่งในลิสต์ของคีย์ (0 คือรายการล่าสุด) ว่างเมื่อลิสต์ไม่ได้ระบุค่านั้น |
| `ValueName` | หมายเลขของค่า (`MRUListEx`) หรือตัวอักษร (`MRUList`) |
| `Program` | โปรแกรมที่แสดงกล่องโต้ตอบ |
| `Path` | ไฟล์หรือโฟลเดอร์: ชื่อของ shell item ที่ต่อกัน หรือพาธที่จัดเก็บเป็นข้อความ |
| `ItemType` | ชนิดของ shell item ตัวสุดท้าย: `file`, `directory`, `root_folder`, `volume`, `network_location`, `property_view`, `unknown`, … |
| `ItemModifiedUTC`, `ItemCreatedUTC`, `ItemAccessedUTC` | เวลาของไฟล์หรือโฟลเดอร์ตามที่ shell item บันทึกไว้ |
| `MFT-Entry`, `MFT-Sequence` | NTFS file reference ที่ shell item บันทึกไว้ |
| `LastUsedUTC` | เวลาเขียนล่าสุดของคีย์ สำหรับรายการล่าสุดเท่านั้น |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์และเวลาเขียนล่าสุด: เวลาของรายการล่าสุด ไม่ใช่ของทุกแถว |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสไม่ได้ หรือรายการ shell item เมื่อชนิดของ item ไม่เป็นที่รู้จัก |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่ลิสต์ระบุแต่คีย์ไม่มี ค่าที่ลิสต์ไม่ได้ระบุ ชนิดที่ไม่ถูกต้อง ชื่อหรือลิสต์ที่ถูกตัดทอน |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. อ่าน `*` ตามลำดับสำหรับผู้ใช้แต่ละคน และดู `LastUsedUTC` สำหรับไฟล์ล่าสุด
2. มองหาสคริปต์ ไฟล์บีบอัด ไฟล์ปฏิบัติการ และเอกสารในตำแหน่งที่ผู้ใช้เขียนได้ แบบถอดได้ หรือบนเครือข่าย
   และไฟล์ที่มีชื่อคล้ายข้อมูลรับรอง (credential) หรือไฟล์ส่งออก
3. ใช้ `LastVisitedPidlMRU` เพื่อเชื่อมโยงโปรแกรมกับโฟลเดอร์ที่ใช้ล่าสุด และใช้ `CIDSizeMRU` และ
   `FirstFolder` สำหรับโปรแกรมที่เคยใช้กล่องโต้ตอบเลย
4. เชื่อมโยงพาธและ `MftEntry` กับ `$MFT`, USN Journal, ไฟล์ LNK, Jump Lists,
   ShellBags และ RecentDocs

## รายละเอียดเพิ่มเติม { #further-details }

- [libyal winreg-kb: คีย์ Most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [libyal libfwsi: รูปแบบ Windows Shell Item](https://github.com/libyal/libfwsi/blob/main/documentation/Windows%20Shell%20Item%20format.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ys-lnk-parser](https://github.com/Yamato-Security/ys-lnk-parser)
