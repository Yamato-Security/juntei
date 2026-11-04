# ประวัติแอปพลิเคชัน { #application-history }

## สิ่งที่เครื่องมือทิ้งร่องรอยไว้ { #what-tools-leave-behind }

เครื่องมือของบุคคลที่สามจำนวนมากเก็บการตั้งค่าและประวัติไว้ใน `NTUSER.DAT` ของผู้ใช้ หลายตัวเป็นเครื่องมือ
ที่ผู้โจมตีใช้ หรือใช้เพื่อเตรียมข้อมูลก่อนนำออกไป ได้แก่ เครื่องมือ Sysinternals โปรแกรมบีบอัดไฟล์
ไคลเอนต์ซิงก์คลาวด์ และเครื่องมือเข้าถึงระยะไกล Juntei อ่านข้อมูลต่อไปนี้:

| แอปพลิเคชัน | คีย์ใต้ `HKCU\Software` | สิ่งที่แสดง |
|---|---|---|
| Sysinternals | `Sysinternals\<tool>` | เครื่องมือแต่ละตัวที่ผู้ใช้ยอมรับสัญญาอนุญาตแล้ว (`EulaAccepted`): PsExec, ProcDump, Sysmon, … |
| WinRAR | `WinRAR\ArcHistory`, `DialogEditHistory\ArcName`, `DialogEditHistory\ExtrPath` | ไฟล์บีบอัดที่เปิด ชื่อไฟล์บีบอัดและโฟลเดอร์ปลายทางการแตกไฟล์ที่พิมพ์ลงในกล่องโต้ตอบ |
| 7-Zip | `7-Zip\Compression\ArcHistory`, `Extraction\PathHistory`, `FM\FolderHistory`, `FM\CopyHistory`, `FM\PanelPath0/1` | ไฟล์บีบอัดที่สร้าง โฟลเดอร์ปลายทางการแตกไฟล์ โฟลเดอร์ที่เรียกดูและคัดลอกไปในตัวจัดการไฟล์ของโปรแกรม |
| WinZip | `Nico Mak Computing\WinZip\filemenu`, `extract`, `mru\archives` | ไฟล์บีบอัดในเมนู File โฟลเดอร์ปลายทางการแตกไฟล์ ไฟล์บีบอัดล่าสุด |
| OneDrive | `Microsoft\OneDrive\Accounts\<account>` | ที่อยู่อีเมล โฟลเดอร์ในเครื่อง และชื่อของแต่ละบัญชี ไลบรารีที่ซิงก์ (`ScopeIdToMountPointPathCache`) และโฟลเดอร์ (`Tenants\<tenant>`) |
| Adobe Acrobat และ Reader | `Adobe\{Acrobat Reader,Adobe Acrobat}\<version>\AVGeneral\cRecentFiles`, `cRecentFolders` | ไฟล์ PDF และโฟลเดอร์ที่เปิดล่าสุด พร้อมชื่อไฟล์ ขนาด และวันที่ |
| TeamViewer | `TeamViewer` (และ `Version<N>` ของเวอร์ชันเก่า) | ค่าทุกค่า: ชื่อบัญชีและชื่อผู้ใช้ โฟลเดอร์เริ่มต้นของการถ่ายโอนไฟล์ (`<partner ID>?<local>|<remote>`), … |
| CCleaner | `Piriform\CCleaner` | ค่าทุกค่า: สิ่งที่ตั้งค่าให้ล้าง, `UpdateKey`, … |

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้รายนี้เรียกใช้เครื่องมือ (หรือยอมรับ
สัญญาอนุญาตของเครื่องมือ) หรือเครื่องมือได้เปิด สร้าง หรือแตกไฟล์บีบอัดและโฟลเดอร์เหล่านี้ หรือบัญชีคลาวด์นี้
ถูกตั้งค่าไว้สำหรับผู้ใช้ ประวัติส่วนใหญ่ไม่มีเวลาของแต่ละรายการ มีเพียงรายการล่าสุดเท่านั้นที่ได้
เวลาเขียนล่าสุด (last-write time) ของคีย์รายการนั้น

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
  Software\Sysinternals\<tool>                         EulaAccepted
  Software\WinRAR\ArcHistory                           0, 1, … (0 the most recent)
  Software\7-Zip\Compression                           ArcHistory (REG_BINARY UTF-16 list)
  Software\Microsoft\OneDrive\Accounts\<account>       UserEmail, UserFolder, …
  Software\Adobe\Acrobat Reader\<version>\AVGeneral\cRecentFiles\c1, c2, …
```

ให้เก็บรวบรวม `NTUSER.DAT` ของผู้ใช้ทุกคนพร้อม transaction log และเก็บ `SOFTWARE` ของระบบเพื่อใช้หา
SID ของผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **การยอมรับ EULA ของ Sysinternals** `FirstUsedUTC` เป็นค่าประมาณเวลาที่ใช้งานครั้งแรกจากเวลาเขียนล่าสุด
  ของคีย์ เมื่อค่าเพียงค่าเดียวของคีย์คือ `EulaAccepted` ที่อ่านได้และตั้งเป็น `1` EULA ที่ถูกปฏิเสธหรือเสียหาย
  จะไม่ได้รับค่าประมาณนี้ การตั้งค่ารีจิสทรียังอาจถูกกำหนดไว้ล่วงหน้า คัดลอก หรือแก้ไขได้ ให้ยืนยันการเรียกใช้งาน
  และเวลาของการเรียกใช้ด้วยอาร์ติแฟกต์อื่น
  เครื่องมือที่เก็บการตั้งค่าไว้ที่นั่น (Process Explorer, Autoruns, …) จะเขียนคีย์ใหม่ในภายหลัง ในกรณีนี้
  `FirstUsedUTC` จะว่าง และ `KeyLastWrittenUTC` เป็นเพียงเวลาที่เปลี่ยนการตั้งค่าครั้งล่าสุด เครื่องมือที่
  เรียกใช้ด้วย `-accepteula` หรือในฐานะผู้ใช้อื่น (ฝั่งระยะไกลของ PsExec ทำงานในฐานะ SYSTEM) จะเขียนคีย์ของผู้ใช้นั้น
- **ลำดับ ไม่ใช่เวลา** รายการในลิสต์มีลำดับ (`MruPosition` 0 คือรายการล่าสุด) แต่มีเพียงเวลาเขียนล่าสุด
  ของคีย์รายการเท่านั้นที่ระบุเวลาของรายการล่าสุด รายการที่เก่ากว่าถูกใช้ก่อนหน้านั้นในเวลาที่ไม่ทราบ
- **ชื่อที่พิมพ์** `ArcName` ของ WinRAR คือชื่อไฟล์บีบอัดที่พิมพ์ตอนสร้าง ซึ่งมักเป็นชื่อไฟล์ที่ไม่มีโฟลเดอร์
- **พาธของ Adobe** จะคงไว้ตามที่ Adobe เขียน (`/C/Users/alice/a.pdf` คือ
  `C:\Users\alice\a.pdf`) ส่วน `sDate` (ใน `Details`) เป็นสตริงวันที่ของ PDF ซึ่งคงไว้ตามที่เขียน
- **`UpdateKey` ของ CCleaner** เป็นเวลาท้องถิ่นที่ไม่มีเขตเวลา และคงไว้ตามที่เขียน
- **TeamViewer** บันทึกการเชื่อมต่อเป็นไฟล์ (`Connections_incoming.txt`, `TeamViewer*_Logfile.log`)
  ไม่ใช่ค่าในรีจิสทรี ให้เก็บรวบรวมไฟล์เหล่านั้นด้วย
- **การไม่มีข้อมูล** ผู้ใช้และเครื่องมือสามารถล้างประวัติเหล่านี้ได้ (CCleaner เองก็ทำได้)

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-history -d evidence/ -o app-history.csv
juntei parse-registry app-history -f NTUSER.DAT -t jsonl \
  | jq 'select(.App == "Sysinternals" and .FirstUsed != null)'
```

สรุปผลจะแสดงค่าประมาณการใช้งานครั้งแรกของ Sysinternals ล่าสุด ไฟล์บีบอัดและโฟลเดอร์ที่แต่ละรายการใช้
ล่าสุด และบัญชี OneDrive

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | โฮสต์และ `NTUSER.DAT` |
| `User`, `UserSID` | ชื่อโฟลเดอร์โปรไฟล์ และ SID ของโปรไฟล์จาก `ProfileList` ของระบบ |
| `App` | `Sysinternals`, `WinRAR`, `7-Zip`, `WinZip`, `OneDrive`, `Adobe Acrobat Reader`, `Adobe Acrobat`, `TeamViewer`, `CCleaner` |
| `Item` | เครื่องมือ Sysinternals, ลิสต์ (`ArcHistory`, `ExtrPath`, `FolderHistory`, `filemenu`, `cRecentFiles`, …), บัญชี OneDrive หรือคีย์ของโฟลเดอร์ที่ซิงก์ หรือชื่อของค่า |
| `MRU-Position` | ตำแหน่งของรายการในลิสต์ โดย 0 คือรายการล่าสุด |
| `Path` | ไฟล์บีบอัด โฟลเดอร์ หรือเอกสาร |
| `Value` | `EulaAccepted`, ที่อยู่อีเมลของบัญชี OneDrive หรือ id ของไลบรารีที่ซิงก์ หรือค่าของ TeamViewer หรือ CCleaner |
| `FirstUsedUTC` | ค่าประมาณการใช้งานครั้งแรกจากการยอมรับ EULA (ดูข้อควรระวังด้านบน) |
| `Details` | `DisplayName`, `UserCID`, `ConfiguredTenantId`, … ของบัญชี OneDrive และ `Version`, `tFileName`, `uFileSize`, `uPageCount`, `sDate` ของเอกสาร Adobe |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName` | แหล่งที่มาของเรคคอร์ด และเวลาที่คีย์นั้นเปลี่ยนแปลงครั้งล่าสุด |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ข้อมูลที่ถอดรหัสหรืออ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. มองหาเครื่องมือ Sysinternals ที่ผู้ใช้ทั่วไปไม่เรียกใช้ (PsExec, ProcDump, PsLoggedOn,
   SDelete) และ timestamp ของ EULA
2. ตรวจทานชื่อไฟล์บีบอัดและโฟลเดอร์ปลายทางการแตกไฟล์ในช่วงเหตุการณ์ เช่น ไฟล์บีบอัดที่สร้างใน
   `ProgramData`, `Temp` หรือโฟลเดอร์ของผู้ใช้ก่อนที่ข้อมูลจะถูกนำออกจากระบบ
3. ตรวจสอบบัญชี OneDrive ที่ไม่ใช่ของบริษัท/องค์กร รวมถึงชื่อบัญชี TeamViewer และโฟลเดอร์ที่ใช้ถ่ายโอนไฟล์
4. เชื่อมโยงกับ Prefetch, Amcache, BAM และ UserAssist (การทำงานของเครื่องมือ), `$MFT` และ USN
   Journal (ไฟล์บีบอัด), ไฟล์ LNK และ Jump Lists (เอกสารที่เปิด) และ event log

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Sysinternals](https://learn.microsoft.com/en-us/sysinternals/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
