# UserAssist { #userassist }

## UserAssist คืออะไร { #what-userassist-is }

Windows Explorer นับโปรแกรมและทางลัดที่ผู้ใช้เริ่มจาก Explorer shell ได้แก่ Start menu, taskbar,
เดสก์ท็อป และการดับเบิลคลิกในโฟลเดอร์ โดยเก็บจำนวนเหล่านี้ไว้ใน `NTUSER.DAT` ของผู้ใช้ แต่ละรายการมี

- จำนวนครั้งที่เรียกใช้
- บน Windows 7 ขึ้นไป จำนวนครั้งและระยะเวลา (มิลลิวินาที) ที่รายการนั้นได้รับโฟกัส
- เวลาที่เรียกใช้ครั้งล่าสุด

ชื่อของค่า (value name) ถูกเข้ารหัสแบบ ROT13 ชื่อของโปรแกรมมักขึ้นต้นด้วย GUID ของ known folder
(`{1AC14E77-…}` คือ `System32`)

UserAssist แสดงสิ่งที่ผู้ใช้รายใดรายหนึ่งเริ่มใช้งานแบบโต้ตอบ ซึ่งหลักฐานการเรียกใช้งานอื่นส่วนใหญ่ไม่ได้ระบุว่าเป็นของผู้ใช้รายใด

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้เริ่มโปรแกรมหรือทางลัดนี้
ผ่าน Explorer เป็นจำนวนครั้งเท่านี้ ครั้งล่าสุด ณ เวลานี้ โปรแกรมที่เริ่มด้วยวิธีอื่น (บรรทัดคำสั่ง บริการ
งานตามกำหนดเวลา หรือโปรแกรมอื่น) จะไม่ถูกนับ

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\UserAssist\{GUID}\Count
```

| GUID | สิ่งที่เก็บ |
|---|---|
| `{CEBFF5CD-ACE2-4F4F-9178-9926F41749EA}` | โปรแกรม (`executable`), Windows 7 ขึ้นไป |
| `{F4E57C4B-2036-45F0-A9AB-443BCFE33D9F}` | ทางลัด (`shortcut`), Windows 7 ขึ้นไป |
| `{75048700-EF1F-11D0-9888-006097DEACF9}` | โปรแกรมและทางลัด (`active_desktop`), Windows 2000 ถึง Vista |
| `{5E6AB780-7743-11CF-A12B-00AA004AE837}` | Internet toolbar (`internet_toolbar`), Windows 2000 ถึง Vista |

GUID อื่น ๆ จะรายงานเป็น `other` ให้เก็บรวบรวม `NTUSER.DAT` พร้อม transaction log ของทุก
โปรไฟล์ รวมถึงไฮฟ์ `SOFTWARE` ของระบบ ซึ่งให้ SID ของผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **โปรไฟล์ใหม่มีรายการตั้งต้นใส่ไว้** บน Windows 7 ขึ้นไป โปรไฟล์ใหม่จะเริ่มต้นด้วยรายการค่าเริ่มต้น
  (Getting Started, Calculator, Paint, Sticky Notes, …) จำนวนครั้งที่เรียกใช้ของรายการเหล่านี้นับถอยหลังจาก 14
  และทั้งหมดใช้เวลาเรียกใช้ล่าสุดร่วมกันเพียงค่าเดียว ซึ่งใกล้กับเวลาที่สร้างโปรไฟล์ รายการเหล่านี้ไม่ใช่
  กิจกรรมของผู้ใช้
- **เวอร์ชัน 3 นับเริ่มจาก 5** (Windows 2000 ถึง Vista) จำนวนที่จัดเก็บเริ่มต้นที่ 5 จึงถูกลบออก 5
  จำนวนที่จัดเก็บซึ่งน้อยกว่า 5 จะรายงานใน `Problems`
- **พาธใช้พาธค่าเริ่มต้นของ known folder** (`%windir%\system32`, `%ProgramFiles%`,
  `%APPDATA%\…`) โฟลเดอร์ที่ถูกเปลี่ยนเส้นทางไปที่อื่นยังคงแสดงพาธค่าเริ่มต้น App ID
  (`Microsoft.Windows.Explorer`, `Microsoft.WindowsNotepad_8wekyb3d8bbwe!App`) และรายการ Control Panel
  (`::{…}`) ไม่มีพาธ
- **การติดตามสามารถถูกปิดได้** ด้วย `Explorer\Advanced` `Start_TrackProgs` = 0,
  `Policies\Explorer` `NoInstrumentation` = 1 หรือ `UserAssist\Settings` `NoLog` = 1 สรุปผล
  จะแสดงรายการไฮฟ์ที่ปิดการติดตามไว้ UserAssist ที่ว่างเปล่าหรือไม่มีอยู่บนโปรไฟล์ที่ใช้งานอยู่อาจบ่งชี้
  การต่อต้านนิติวิทยาศาสตร์ (anti-forensics)
- **`NoEncrypt` = 1** ทำให้ Windows เขียนชื่อโดยไม่ใช้ ROT13 ชื่อดังกล่าวจะถอดรหัสผิด ดังนั้น
  ทุกรายการของไฮฟ์นั้นจะระบุเรื่องนี้ไว้ใน `Problems` (และใน stderr เมื่อเรคคอร์ดถูกส่งไปยัง stdout)
- **เวลาเรียกใช้ล่าสุดและจำนวนครั้งถูกเก็บแยกตามรายการ** แต่เวลาเขียนล่าสุดของคีย์
  (`KeyLastWrittenUTC`) เป็นเพียงเวลาของการอัปเดตล่าสุดต่อรายการใดก็ได้
- **จำนวนครั้งที่เรียกใช้อาจเป็น 0** สำหรับรายการที่เพียงได้รับโฟกัส จำนวนครั้งและเวลาที่ได้รับโฟกัสมีอยู่เฉพาะใน
  เวอร์ชัน 5 (Windows 7 ขึ้นไป)

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry user-assist <INPUT> [OPTIONS]

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
juntei parse-registry user-assist -d evidence/ -o userassist.csv
juntei parse-registry user-assist -f NTUSER.DAT -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

สรุปผลจะแสดงรายการที่เรียกใช้ล่าสุด (ใหม่สุดก่อน) พร้อมผู้ใช้ และไฮฟ์ที่ปิด
UserAssist ไว้

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ ชื่อโฟลเดอร์โปรไฟล์ SID ของผู้ใช้จาก `ProfileList` ของระบบ (เมื่อมีไฮฟ์ `SOFTWARE` ของระบบ) และไฮฟ์ |
| `GUID`, `Kind` | คีย์ `{GUID}` และสิ่งที่คีย์นั้นเก็บ (`executable`, `shortcut`, `active_desktop`, `internet_toolbar`, `other`) |
| `NameRot13`, `Name` | ชื่อของค่าตามที่จัดเก็บ และที่ถอดรหัสแล้ว |
| `Path` | ชื่อที่ถอดรหัสแล้วในรูปพาธ โดยแทนที่ GUID ของ known folder ด้วยพาธค่าเริ่มต้น ว่างเปล่าสำหรับ App ID และ shell item |
| `RunCount` | จำนวนครั้งที่เริ่มจาก Explorer (เวอร์ชัน 3: จำนวนที่จัดเก็บลบ 5) |
| `FocusCount`, `FocusTimeMs` | จำนวนครั้งที่ได้รับโฟกัส และระยะเวลาเป็นมิลลิวินาที (เวอร์ชัน 5) |
| `LastRunUTC` | เวลาที่เริ่มครั้งล่าสุด ว่างเปล่าเมื่อ Windows จัดเก็บค่า 0 หรือเมื่อเวลาที่จัดเก็บเกินปี 9999 (ในกรณีนั้น `Problems` จะระบุค่าไว้ และ `DataRaw` เก็บไบต์ไว้) |
| `Format`, `SessionID` | รูปแบบของข้อมูล (5: 72 ไบต์, 3: 16 ไบต์) และหมายเลขเซสชันที่เขียนครั้งล่าสุด |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ `Count` คือการอัปเดตล่าสุดต่อรายการใดก็ได้ในคีย์นั้น |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ข้อมูลที่มีขนาดไม่รู้จัก ขนาดที่ไม่ตรงกับ `Version` ของคีย์ จำนวนของเวอร์ชัน 3 ที่น้อยกว่า 5 เวลาเรียกใช้ล่าสุดที่เกินปี 9999 และ `NoEncrypt` |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงลำดับตาม `LastRunUTC` และแยกรายการตั้งต้นออก รายการเหล่านี้ใช้เวลาเดียวกัน ซึ่งใกล้กับเวลาที่
   สร้างโปรไฟล์
2. ค้นหาโปรแกรมในตำแหน่งที่ผู้ใช้เขียนได้ (`%USERPROFILE%`, `Downloads`, `Temp`,
   `ProgramData`) ไดรฟ์แบบถอดได้ และพาธเครือข่าย รวมถึงเครื่องมือดูแลระบบและเครื่องมือเข้าถึง
   ระยะไกล
3. เชื่อมโยงกับ Prefetch (การเรียกใช้งานพร้อมเวลาที่เรียกใช้), Amcache และ ShimCache (การมีอยู่),
   Jump Lists และไฟล์ LNK (สิ่งที่ถูกเปิดด้วยโปรแกรมเหล่านั้น) และ `$MFT` สำหรับไฟล์โปรแกรม
4. ตรวจสอบไฮฟ์ที่ปิดการติดตามไว้ และโปรไฟล์ที่ UserAssist ว่างเปล่า เพื่อหาร่องรอยการล้างข้อมูล
5. ใช้ `parse-registry read --recover-deleted` สำหรับค่า UserAssist ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: KNOWNFOLDERID](https://learn.microsoft.com/en-us/windows/win32/shell/knownfolderid)
- [libyal winreg-kb: คีย์ User Assist](https://github.com/libyal/winreg-kb/blob/main/documentation/User%20Assist%20keys.asciidoc)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
