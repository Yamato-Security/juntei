# รายการ MRU ที่ใช้ล่าสุด (RunMRU, TypedPaths, WordWheelQuery, TypedURLs) { #most-recently-used-lists-runmru-typedpaths-wordwheelquery-typedurls }

## รายการเหล่านี้คืออะไร { #what-the-lists-are }

Windows และโปรแกรมต่าง ๆ จะเก็บรายการสั้น ๆ ของสิ่งที่ผู้ใช้พิมพ์หรือเปิดล่าสุดไว้ใน
`NTUSER.DAT` ของผู้ใช้:

- **RunMRU:** คำสั่งที่พิมพ์ในกล่องโต้ตอบ Run (Win+R): `cmd`, `powershell -ep bypass`,
  `\\server\share`, `regedit`
- **TypedPaths:** พาธที่พิมพ์ในแถบที่อยู่ของ Explorer
- **WordWheelQuery:** คำที่ค้นหาในช่องค้นหาของ Explorer
- **TypedURLs:** ที่อยู่ที่พิมพ์ในแถบที่อยู่ของ Internet Explorer และตั้งแต่ Windows 8 มี
  `TypedURLsTime` ซึ่งระบุเวลาที่พิมพ์แต่ละรายการ
- **Paint และ WordPad:** ไฟล์ที่แต่ละโปรแกรมเปิดล่าสุด (`Recent File List`)
- **ACMru (Windows XP):** คำค้นของ Search Assistant แยกตามชนิดการค้นหา

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้พิมพ์คำสั่ง พาธ
คำค้น หรือที่อยู่นี้ หรือโปรแกรมได้เปิดไฟล์นี้ มีเพียงรายการล่าสุดของแต่ละรายการเท่านั้น
ที่ระบุวันเวลาได้ โดยใช้เวลาเขียนล่าสุดของคีย์ ยกเว้นที่อยู่ที่พิมพ์ ซึ่ง `TypedURLsTime` ระบุเวลาไว้

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RunMRU          a, b, … in MRUList order
Software\Microsoft\Windows\CurrentVersion\Explorer\TypedPaths      url1 (the most recent), url2, …
Software\Microsoft\Windows\CurrentVersion\Explorer\WordWheelQuery  0, 1, … in MRUListEx order
Software\Microsoft\Internet Explorer\TypedURLs                     url1, url2, …
Software\Microsoft\Internet Explorer\TypedURLsTime                 url1, url2, … (FILETIME)
Software\Microsoft\Windows\CurrentVersion\Applets\Paint\Recent File List    File1, File2, …
Software\Microsoft\Windows\CurrentVersion\Applets\Wordpad\Recent File List  File1, File2, …
Software\Microsoft\Search Assistant\ACMru\<kind>                   000, 001, … (Windows XP)
```

ซับคีย์ของ `ACMru` คือชนิดการค้นหา: `5603` ชื่อไฟล์และโฟลเดอร์ `5604` คำหรือวลีใน
ไฟล์ `5647` คอมพิวเตอร์และบุคคล `5001` อินเทอร์เน็ต

ให้เก็บรวบรวม `NTUSER.DAT` พร้อม transaction log ของทุกโปรไฟล์

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **หนึ่งเวลาต่อหนึ่งรายการ** เวลาเขียนล่าสุดของคีย์จะเปลี่ยนเมื่อรายการเปลี่ยน จึงระบุเวลาได้เฉพาะ
  รายการล่าสุดเท่านั้น (`IsMostRecent`, `LastUsedUTC`) รายการอื่นเรียงตามลำดับแต่ไม่มีวันเวลา
  ส่วน `TypedURLsTime` ระบุเวลาของที่อยู่ที่พิมพ์แต่ละรายการ (`TypedTimeUTC`)
- **พิมพ์ ไม่ใช่เรียกใช้** RunMRU บันทึกสิ่งที่พิมพ์ในกล่องโต้ตอบ Run ไม่ว่าจะเรียกใช้สำเร็จหรือไม่
  Windows จัดเก็บแต่ละคำสั่งโดยมี `\1` ต่อท้าย ซึ่ง Juntei จะตัดออก ให้เชื่อมโยง
  กับ Prefetch, BAM, UserAssist และเหตุการณ์การสร้างโพรเซส
- **รายการสั้น** แต่ละรายการเก็บเฉพาะรายการล่าสุดเพียงไม่กี่รายการ และการตั้งค่าความเป็นส่วนตัวหรือเครื่องมือทำความสะอาด
  สามารถล้างรายการเหล่านี้ได้ ใช้ `parse-registry read --recover-deleted` สำหรับค่าที่ถูกลบ
- **คำค้นคือสิ่งที่พิมพ์** WordWheelQuery และ ACMru เก็บข้อความที่พิมพ์ ไม่ใช่สิ่งที่ค้นพบ
- **ลำดับมาจากรายการ** `MRUList` ของ RunMRU และ `MRUListEx` ของ WordWheelQuery ให้ลำดับ
  ส่วนรายการอื่น ลำดับมาจากหมายเลขของค่า (`url1`, `File1` และ `000` คือรายการล่าสุด)

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry mru <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find NTUSER.DAT hives
  -f, --file <FILE>      Parse one NTUSER.DAT
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --list <LISTS>  runmru, typedpaths, wordwheelquery, typedurls, paint, wordpad, acmru

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry mru -d evidence/ -o mru.csv
juntei parse-registry mru -f NTUSER.DAT --list wordwheelquery,typedpaths -t jsonl
juntei parse-registry mru -f NTUSER.DAT --list runmru -t jsonl \
  | jq 'select(.Value | test("powershell|cmd|mshta"; "i"))'
```

สรุปผลจะแสดงจำนวนรายการในแต่ละรายการ และรายการที่มีวันเวลาล่าสุด (ที่อยู่ที่พิมพ์ และรายการล่าสุด
ของแต่ละรายการ) โดยเรียงจากใหม่ไปเก่า

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `User`, `UserSID`, `SourcePath` | โฮสต์ เจ้าของโปรไฟล์ (จากพาธหรือ shell folder) SID จาก `ProfileList` ของระบบ และไฮฟ์ |
| `List` | `RunMRU`, `TypedPaths`, `WordWheelQuery`, `TypedURLs`, `Paint`, `Wordpad` หรือ `ACMru` |
| `Subkey` | ซับคีย์ของ `ACMru`: ชนิดการค้นหา (`5603`, `5604`, `5647`, `5001`) |
| `MRU-Position`, `IsMostRecent` | ตำแหน่งในรายการ (0 คือล่าสุด) ว่างเปล่าเมื่อรายการไม่ได้ระบุชื่อค่านั้น |
| `ValueName` | `a`, `0`, `url1`, `File1`, `000`, … |
| `Value` | คำสั่ง (ไม่รวม `\1`) พาธ คำค้น ที่อยู่ หรือไฟล์ |
| `TypedTimeUTC` | เวลาที่พิมพ์ที่อยู่ จาก `TypedURLsTime` |
| `LastUsedUTC` | เวลาเขียนล่าสุดของคีย์ สำหรับรายการล่าสุดเท่านั้น |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์และเวลาเขียนล่าสุดของคีย์: เป็นเวลาของรายการล่าสุด ไม่ใช่ของทุกแถว |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อถอดรหัสไม่ได้ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ค่าที่รายการระบุชื่อแต่คีย์ไม่มี ค่าที่รายการไม่ได้ระบุชื่อ ค่าที่ตั้งชื่อผิด ชนิดข้อมูลผิด คำค้นที่ถูกตัดทอน และเวลาที่ไม่ใช่ FILETIME |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. อ่าน RunMRU เพื่อหาคำสั่ง สคริปต์ แชร์บนเครือข่าย และเครื่องมือดูแลระบบที่ผู้ใช้พิมพ์
2. อ่าน TypedPaths และ WordWheelQuery เพื่อหาโฟลเดอร์และไฟล์ที่ผู้ใช้ค้นหา:
   ข้อมูลประจำตัว (credentials) ไฟล์ส่งออก ไฟล์บีบอัด ตำแหน่งบนสื่อแบบถอดได้และบนเครือข่าย
3. นำที่อยู่ที่พิมพ์ไปวางบนไทม์ไลน์ด้วย `TypedTimeUTC`
4. เชื่อมโยงกับ Prefetch, BAM, UserAssist, ShellBags, ไฟล์ LNK, Jump Lists และประวัติเบราว์เซอร์

## รายละเอียดเพิ่มเติม { #further-details }

- [libyal winreg-kb: คีย์ Most recently used (MRU)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/explorer-keys/Most-recently-used.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
