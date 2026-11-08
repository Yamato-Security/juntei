# เว็บเบราว์เซอร์ { #web-browsers }

## ฐานข้อมูลของเบราว์เซอร์คืออะไร { #what-the-browser-databases-are }

เบราว์เซอร์เก็บประวัติ การดาวน์โหลด คุกกี้ และข้อมูลที่กรอกในฟอร์มไว้ในฐานข้อมูล SQLite ภายในโปรไฟล์ของ
ผู้ใช้แต่ละคน เบราว์เซอร์ที่ใช้ Chromium เป็นพื้นฐาน (Chrome, Edge, Brave, Opera, Vivaldi) และเบราว์เซอร์
ที่ใช้ Firefox เป็นพื้นฐานต่างก็ใช้ไฟล์และตารางของตนเอง แต่บันทึกกิจกรรมประเภทเดียวกัน ได้แก่

- **ประวัติ**: หน้าเว็บที่เข้าชม การเข้าชมแต่ละครั้งพร้อมเวลา วิธีที่เบราว์เซอร์มาถึงหน้านั้น (พิมพ์ URL
  ลิงก์ การเปลี่ยนเส้นทาง บุ๊กมาร์ก) และใน Chromium จะมีระยะเวลาที่หน้านั้นอยู่ด้านหน้าด้วย
- **การดาวน์โหลด**: URL ที่ไฟล์ดาวน์โหลดมา (และการเปลี่ยนเส้นทางระหว่างทาง) ตำแหน่งที่บันทึกไฟล์ ขนาด
  เวลาที่การดาวน์โหลดเริ่มต้นและสิ้นสุด และการดาวน์โหลดเสร็จสมบูรณ์หรือไม่
- **คุกกี้**: ไซต์ที่ตั้งค่าคุกกี้ เวลาที่ตั้งค่า เวลาที่ส่งครั้งล่าสุด และเวลาหมดอายุ
- **ประวัติฟอร์ม**: ค่าที่พิมพ์ในฟอร์มบนเว็บและช่องค้นหา จำนวนครั้ง และเวลาครั้งแรกและครั้งล่าสุด

กิจกรรมของเบราว์เซอร์มักเป็นบันทึกที่ตรงที่สุดว่าผู้ใช้ทำอะไร เช่น หน้าฟิชชิงที่ผู้ใช้เปิดตาม เครื่องมือที่
ดาวน์โหลด เว็บเมลที่ใช้ และคำค้นหาที่พิมพ์

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรไฟล์เบราว์เซอร์บันทึกการเข้าชม URL นั้น
ณ เวลาที่บันทึกไว้ หรือบันทึกไฟล์ที่ดาวน์โหลดไว้ที่พาธที่บันทึกไว้ ฐานข้อมูลไม่ได้บอกว่าใครเป็นผู้ใช้งาน
เครื่องอยู่ในขณะนั้น

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\History
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Network\Cookies
%LOCALAPPDATA%\Google\Chrome\User Data\<Profile>\Web Data
%LOCALAPPDATA%\Microsoft\Edge\User Data\<Profile>\...
%LOCALAPPDATA%\BraveSoftware\Brave-Browser\User Data\<Profile>\...
%APPDATA%\Opera Software\Opera Stable\...
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\places.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\cookies.sqlite
%APPDATA%\Mozilla\Firefox\Profiles\<profile>\formhistory.sqlite
```

`<Profile>` คือ `Default`, `Profile 1`, `Profile 2` และต่อไปเรื่อย ๆ Chromium ย้าย `Cookies` ไปไว้ใน
`Network` ตั้งแต่เวอร์ชัน 96 ส่วนเวอร์ชันก่อนหน้าเก็บไว้ข้าง `History` Firefox จนถึงเวอร์ชัน 25 เก็บ
การดาวน์โหลดไว้ใน `downloads.sqlite` ส่วนเวอร์ชันหลังจากนั้นเก็บไว้ใน `places.sqlite`

ให้เก็บรวบรวมทุกไฟล์ของโปรไฟล์ และเก็บแต่ละฐานข้อมูลไปพร้อมกับไฟล์ที่อยู่ข้างกันซึ่งขึ้นต้นด้วยชื่อเดียวกัน
ได้แก่ `places.sqlite-wal`, `cookies.sqlite-wal`, `formhistory.sqlite-wal`, `History-journal`

!!! warning "บันทึกแบบ write-ahead"
    Firefox เก็บฐานข้อมูลในโหมด write-ahead log: การเปลี่ยนแปลงล่าสุดจะอยู่ในไฟล์ `-wal` เท่านั้น
    จนกว่าเบราว์เซอร์จะคัดลอกการเปลี่ยนแปลงเหล่านั้นลงในฐานข้อมูล (checkpoint) Juntei อ่านแต่ละฐานข้อมูล
    พร้อมไฟล์ `-wal` ของฐานข้อมูลนั้น เช่นเดียวกับที่เบราว์เซอร์ทำ หากไม่มีไฟล์นี้ การเข้าชม คุกกี้ และ
    รายการฟอร์มล่าสุดจะขาดหายไป และรายการที่ถูกลบในบันทึกจะยังถูกอ่านเป็นรายการที่ยังใช้งานอยู่

## รายการที่ถูกลบ { #deleted-entries }

เมื่อเบราว์เซอร์ลบประวัติ แถวจะถูกนำออกจากตาราง แต่เรคคอร์ดของแถวเหล่านั้นอาจยังคงอยู่ในพื้นที่ว่างของ
ฐานข้อมูล ใน freelist หรือในเพจเวอร์ชันเก่าของฐานข้อมูล ได้แก่ ในไฟล์ `-wal` หรือในไฟล์ฐานข้อมูลเองซึ่งยัง
เก็บเพจเวอร์ชันเดิมที่ `-wal` แทนที่ไปแล้ว Juntei กู้คืนการเข้าชม หน้าเว็บ การดาวน์โหลด และรายการฟอร์ม
ที่ถูกลบจากแหล่งเหล่านี้ เป็นแถวที่มี `Status` เป็น `Deleted`

- `Confidence` บอกระดับความเชื่อมั่นว่าเรคคอร์ดนั้นตรงกับตารางของมัน: `High` สำหรับเรคคอร์ดที่สมบูรณ์ซึ่ง
  ทุกค่ามีชนิดตรงกับคอลัมน์ของตน `Medium` หรือ `Low` สำหรับเรคคอร์ดที่เสียหาย หรือเรคคอร์ดที่เข้าได้กับ
  หลายตาราง
- `RecoveredFrom` บอกว่าเรคคอร์ดอยู่ที่ใด: ส่วนของเพจ (cell, freeblock, พื้นที่ที่ไม่ได้จัดสรร, เพจว่าง)
  เพจใดและเวอร์ชันใดของเพจนั้น และออฟเซ็ตของเรคคอร์ดภายในเพจ โดยเฟรม WAL นับเริ่มจาก 0
- `LostColumns` แสดงรายการคอลัมน์ที่เรคคอร์ดที่เสียหายไม่มีข้อมูลอยู่แล้ว ฟิลด์ของคอลัมน์เหล่านั้นจะว่างเปล่า

เบราว์เซอร์ลบข้อมูลโดยใช้ `secure_delete` ของ SQLite ซึ่งเขียนศูนย์ทับพื้นที่ที่ถูกปล่อยในฐานข้อมูล สิ่งที่
กู้คืนได้จึงส่วนใหญ่อยู่ในเพจเวอร์ชันเก่า ได้แก่ เวอร์ชันที่ไฟล์ `-wal` ยังเก็บไว้ และสำเนาเพจในไฟล์ฐานข้อมูล
เองที่ `-wal` แทนที่ไปแล้ว จนกว่าจะเกิด checkpoint ครั้งถัดไป ให้คัดลอกไฟล์ `-wal` ไปพร้อมกับฐานข้อมูล
และทำก่อนที่เบราว์เซอร์จะถูกเปิดใช้งานอีกครั้ง

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- เวลาเป็น UTC Chromium และ Firefox บันทึกเวลาละเอียดถึงระดับไมโครวินาที ส่วนประวัติฟอร์มของ Chromium
  ละเอียดถึงระดับวินาที
- การเข้าชมไม่ได้หมายความว่าผู้ใช้พิมพ์หรือคลิก การเปลี่ยนเส้นทาง เฟรม และหน้าที่โปรแกรมอื่นเปิดก็นับเป็น
  การเข้าชมเช่นกัน ให้ตรวจสอบ `Transition`, `Typed` และ `Hidden`
- Chromium เข้ารหัสค่าของคุกกี้ ดังนั้น `Value` ของคุกกี้เหล่านั้นจึงว่างเปล่า Juntei ไม่ถอดรหัสค่าเหล่านี้
- แถวประวัติฟอร์มที่เป็น `Removed` มาจาก `moz_deleted_formhistory` ของ Firefox ซึ่ง Firefox for
  Android บันทึกไว้สำหรับ Sync โดยปกติ Firefox บนเดสก์ท็อปจะปล่อยตารางนี้ว่างไว้
- รายการที่ถูกลบซึ่งมีระดับความเชื่อมั่น `Low` อาจเป็นเศษของเรคคอร์ดอื่น ให้ตรวจสอบเทียบกับแถวที่ยังใช้งาน
  อยู่และหลักฐานอื่น
- การไม่มีแถวที่ถูกลบไม่ได้หมายความว่าไม่มีสิ่งใดถูกลบ พื้นที่ที่ถูกเขียนศูนย์ทับและ checkpoint จะไม่เหลือ
  สิ่งใดให้กู้คืน และปัจจุบันยังไม่กู้คืนคุกกี้ที่ถูกลบ
- เนื้อหาฐานข้อมูลกำหนดตระกูลเบราว์เซอร์ พาธโปรไฟล์ที่สอดคล้องกันช่วยระบุ `Browser` เป็น Chrome หรือ Firefox แต่พาธที่ขัดแย้งกันจะไม่เปลี่ยนตระกูล มิฉะนั้นใช้ `Chromium-based`, `Firefox-based` หรือ `Safari` (`Unknown` หากระบุไม่ได้) ส่วน `User` และ `Profile` มาจากพาธและอาจสูญหายเมื่อคัดลอกไฟล์
- ไม่อ่าน rollback journal (`-journal`) รวมถึง `WebCacheV01.dat` ของ Internet Explorer และ Edge
  รุ่นเก่า (legacy Edge)

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-sqlite browser <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find History, Cookies, Web Data, places.sqlite, ...
  -f, --file <FILE>      Parse one browser database (and its -wal file)

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-sqlite browser -d Users -o browser-csv
juntei parse-sqlite browser -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, Browser, User}'
```

`-d` ค้นหาไฟล์ตามชื่อและแยกแยะแต่ละไฟล์จากตารางที่อยู่ภายใน ไฟล์ `History` ของโปรแกรมอื่นจะถูกข้ามไป
ส่วน `-f` อ่านฐานข้อมูลไม่ว่าไฟล์จะมีชื่ออะไร

## ตารางเอาต์พุต { #output-tables }

CSV จะสร้างสี่ไฟล์ โดยแต่ละไฟล์มีเพียงแถวส่วนหัวเมื่อไม่มีแถวข้อมูล แถว JSON และ JSONL มี `Table`
(`History`, `Downloads`, `Cookies` หรือ `FormHistory`) ต่อจาก `SourceFile`

| ไฟล์ CSV | เนื้อหา |
|---|---|
| `Browser-History.csv` | การเข้าชม ทั้งที่ยังใช้งานอยู่และที่ถูกลบ รวมถึงหน้าเว็บที่ถูกลบซึ่ง URL ไม่ตรงกับหน้าเว็บที่ยังใช้งานอยู่ใด ๆ |
| `Browser-Downloads.csv` | การดาวน์โหลด ทั้งที่ยังใช้งานอยู่และที่ถูกลบ |
| `Browser-Cookies.csv` | คุกกี้ |
| `Browser-FormHistory.csv` | รายการฟอร์ม ทั้งที่ยังใช้งานอยู่และที่ถูกลบ รวมถึงเรคคอร์ดการนำออก (removal record) ของ Firefox |

## ฟิลด์ { #fields }

ทุกตารางขึ้นต้นด้วยฟิลด์เหล่านี้

| ฟิลด์ | การตีความ |
|---|---|
| `SourceFile` | ฐานข้อมูลที่อ่านแถวนั้นมา |
| `User` | ผู้ใช้ Windows ที่เป็นเจ้าของโปรไฟล์ซึ่งพาธนั้นอยู่ (โฟลเดอร์ถัดจาก `Users`) |
| `Browser` | ตระกูลจากเนื้อหา ระบุชื่อผลิตภัณฑ์เพิ่มด้วยพาธโปรไฟล์ที่สอดคล้องกัน |
| `Profile` | โปรไฟล์ของเบราว์เซอร์: โฟลเดอร์ของฐานข้อมูล |
| `Status` | `Live`, `Deleted` (กู้คืนแล้ว) หรือ `Removed` (เรคคอร์ดการนำออกของ Firefox) |

### ประวัติ { #history }

| ฟิลด์ | การตีความ |
|---|---|
| `RecordType` | `Visit` หรือ `Page` สำหรับหน้าเว็บที่ถูกลบซึ่งการเข้าชมของหน้านั้นอาจหายไปด้วย |
| `VisitTime` | เวลาที่การเข้าชมเริ่มต้น (UTC) |
| `URL`, `Title` | หน้าเว็บและชื่อเรื่องของหน้า ตามที่พบครั้งล่าสุด |
| `Transition` | วิธีที่เบราว์เซอร์มาถึงหน้านั้น Chromium: ประเภทหลักตามด้วย qualifier (`LINK`, `TYPED`, `AUTO_BOOKMARK`, `FORM_SUBMIT`, `RELOAD`, … พร้อม `CHAIN_START`, `SERVER_REDIRECT`, `FROM_ADDRESS_BAR`, …) Firefox: `LINK`, `TYPED`, `BOOKMARK`, `EMBED`, `REDIRECT_PERMANENT`, `REDIRECT_TEMPORARY`, `DOWNLOAD`, `FRAMED_LINK`, `RELOAD` |
| `VisitDuration` | จำนวนวินาทีที่หน้านั้นอยู่ด้านหน้า (Chromium; 0 เมื่อไม่ได้วัด) |
| `VisitCount` | จำนวนการเข้าชมหน้านั้นทั้งหมด |
| `Typed`, `TypedCount` | URL เคยถูกพิมพ์หรือไม่ และกี่ครั้ง (Chromium) |
| `Hidden` | ซ่อนจากรายการประวัติ ได้แก่ subframe ต้นทางของการเปลี่ยนเส้นทาง และหน้าข้อผิดพลาด |
| `Frecency` | การจัดอันดับหน้าของ Firefox สำหรับคำแนะนำในแถบที่อยู่ |
| `VisitID`, `FromVisitID` | row id ของการเข้าชม และของการเข้าชมที่นำมาสู่การเข้าชมนี้ (หน้าที่อ้างอิง หรือต้นทางของการเปลี่ยนเส้นทาง) row id ของการเข้าชมจะว่างเปล่าเมื่อเรคคอร์ดที่กู้คืนสูญเสียค่านี้ไป |
| `LastVisitTime` | การเข้าชมครั้งล่าสุดของหน้าเว็บที่ถูกลบ (UTC) |
| `PageSource` | แหล่งที่มาของ URL และชื่อเรื่องของการเข้าชมที่ถูกลบ: `Live` (หน้าเว็บที่ยังใช้งานอยู่), `Recovered` (หน้าเว็บที่ถูกลบ) หรือ `NotFound` |
| `Confidence`, `RecoveredFrom`, `LostColumns` | วิธีที่รายการที่ถูกลบถูกกู้คืน ดู [รายการที่ถูกลบ](#deleted-entries) |

### การดาวน์โหลด { #downloads }

| ฟิลด์ | การตีความ |
|---|---|
| `StartTime`, `EndTime` | เวลาที่การดาวน์โหลดเริ่มต้นและสิ้นสุด (UTC) |
| `URL`, `RedirectChain` | แหล่งที่มาของไฟล์ และทุก URL ระหว่างทาง (Chromium; ใน CSV คั่นด้วย ` -> `) |
| `TargetPath`, `CurrentPath` | ตำแหน่งที่บันทึกไฟล์ (เป็น URI `file://` ใน Firefox) และตำแหน่งที่เขียนไฟล์ระหว่างดาวน์โหลด |
| `ReceivedBytes`, `TotalBytes` | จำนวนไบต์ที่ได้รับ และขนาดของไฟล์ |
| `State` | `Complete`, `InProgress`, `Cancelled`, `Interrupted`, `Paused`, `Blocked` หรือ `Dirty` (ถูกบล็อกเพราะเป็นมัลแวร์) |
| `DangerType` | คำตัดสินของ Chromium ต่อไฟล์: 0 ไม่อันตราย, 1 ชนิดไฟล์อันตราย, 2 URL อันตราย, 3 เนื้อหาอันตราย, 4 อาจเป็นอันตราย, 5 ไม่ค่อยพบ, 6 ผู้ใช้เลือกเก็บไว้, 7 โฮสต์อันตราย, 8 อาจเป็นซอฟต์แวร์ที่ไม่พึงประสงค์, … |
| `InterruptReason` | สาเหตุที่การดาวน์โหลดที่ถูกขัดจังหวะหยุดลง (Chromium; 0 เมื่อไม่มี) |
| `Referrer`, `TabURL` | หน้าที่อ้างอิง และหน้าในแท็บที่เริ่มการดาวน์โหลด |
| `MimeType`, `Opened` | ชนิดของไฟล์ตามที่เซิร์ฟเวอร์ส่งมา และไฟล์ถูกเปิดจากเบราว์เซอร์หรือไม่ (Chromium) |
| `FileDeletedByUser` | ผู้ใช้ลบไฟล์ที่ดาวน์โหลดออกจากดิสก์ผ่านรายการดาวน์โหลดของเบราว์เซอร์หรือไม่ (Firefox) |
| `DownloadID` | row id ของการดาวน์โหลด ว่างเปล่าเมื่อเรคคอร์ดที่กู้คืนสูญเสียค่านี้ไป |

### คุกกี้ { #cookies }

| ฟิลด์ | การตีความ |
|---|---|
| `Created`, `LastAccessed`, `Expires` | เวลาที่ตั้งค่าคุกกี้ เวลาที่ส่งครั้งล่าสุด และเวลาหมดอายุ (UTC; `Expires` ว่างสำหรับ session cookie) |
| `Host`, `Name`, `Path` | ไซต์ที่คุกกี้นั้นใช้ ชื่อ และพาธของคุกกี้ |
| `Value` | ค่าตามที่จัดเก็บไว้ ซึ่งว่างเปล่าเมื่อ Chromium เข้ารหัสไว้ |
| `Secure`, `HttpOnly`, `Persistent` | ส่งผ่าน HTTPS เท่านั้น, ซ่อนจากสคริปต์, เก็บไว้ข้ามเซสชัน (Chromium) |
| `GoogleAnalytics` | ฟิลด์ที่ถอดรหัสแล้วของคุกกี้ Google Analytics (`__utma`, `__utmb`, `__utmt`, `__utmz`): เวลาการเข้าชม เซสชัน รวมถึงแหล่งที่มาของแคมเปญและคำค้นหา |
| `RowID` | row id ของคุกกี้ |

### ประวัติฟอร์ม { #form-history }

| ฟิลด์ | การตีความ |
|---|---|
| `FirstUsed`, `LastUsed` | เวลาที่ป้อนค่านั้นครั้งแรกและครั้งล่าสุด (UTC) |
| `RemovedTime` | เวลาที่เรคคอร์ดการนำออกของ Firefox ระบุว่ารายการถูกนำออก (UTC) |
| `FieldName`, `Value` | ฟิลด์ของฟอร์ม (`email`, `q`, แถบค้นหาของ Firefox: `searchbar-history`) และสิ่งที่พิมพ์ |
| `Count` | จำนวนครั้งที่ป้อนค่านั้น |
| `GUID` | ตัวระบุรายการของ Firefox ซึ่งเรคคอร์ดการนำออกอ้างถึง |
| `Sources` | แหล่งที่ Firefox บันทึกรายการนั้นมา ในเวอร์ชันที่เก็บข้อมูลนี้ (ใน CSV คั่นด้วยเส้นตั้ง) |
| `RowID` | ID แถวของรายการ ว่าง/null หากสูญหายระหว่างการกู้คืน ค่าเลขศูนย์ที่บันทึกจริงยังคงเป็น `0` |

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียนตารางลงใน `BrowserActivity` เป็น `Browser-History.csv`,
`Browser-Downloads.csv`, `Browser-Cookies.csv` และ `Browser-FormHistory.csv` (เป็นไฟล์ `Browser.json` หรือ
`Browser.jsonl` ไฟล์เดียวเมื่อใช้ `-t json` หรือ `-t jsonl`) พร้อม `BrowserActivity/Browser-logs.txt`
ไทม์ไลน์แบบรวม `-j` จะรวมทุกแถว พร้อม `Comp` และตัวระบุ `Artifact` ที่เป็น `browser` ส่วน `-Z`
จะแปลง `VisitTime`, `LastVisitTime`, `StartTime`, `EndTime`, `Created`, `LastAccessed`, `Expires`,
`FirstUsed`, `LastUsed` และ `RemovedTime`

## รายละเอียดเพิ่มเติม { #further-details }

- [Chromium — page transition types](https://source.chromium.org/chromium/chromium/src/+/main:ui/base/page_transition_types.h)
- [Chromium — download danger types](https://source.chromium.org/chromium/chromium/src/+/main:components/download/public/common/download_danger_type.h)
- [Mozilla — Places database](https://firefox-source-docs.mozilla.org/browser/places/)
- [SQLite — write-ahead logging](https://www.sqlite.org/wal.html)
- [Plaso — browser SQLite parsers](https://github.com/log2timeline/plaso/tree/main/plaso/parsers/sqlite_plugins)
- [Velociraptor — SQLiteHunter](https://github.com/Velocidex/SQLiteHunter)
- [Hindsight](https://github.com/obsidianforensics/hindsight)
