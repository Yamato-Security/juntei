# Internet Explorer และ Edge รุ่นเก่า { #internet-explorer-and-legacy-edge }

## ฐานข้อมูล WebCache คืออะไร { #what-the-webcache-database-is }

`WebCacheV01.dat` คือฐานข้อมูลของฟังก์ชันอินเทอร์เน็ตของ Windows (WinINet) ในโปรไฟล์ของผู้ใช้แต่ละคน
ซึ่งเป็นฐานข้อมูล ESE Internet Explorer 10 และ 11 เก็บประวัติ การดาวน์โหลด แคช และคุกกี้ไว้ในฐานข้อมูลนี้
เช่นเดียวกับ Edge รุ่นเก่า (legacy Edge) ซึ่งเป็น Edge ของ Windows 10 ก่อนที่ Edge จะเปลี่ยนมาใช้ Chromium
เป็นพื้นฐาน เรคคอร์ดแต่ละประเภทอยู่ในคอนเทนเนอร์ (container) ซึ่งเป็นตารางแยกของตนเอง ได้แก่

- **ประวัติ** (`History` และ `MSHist01…` สำหรับช่วงหนึ่งวันหรือหนึ่งสัปดาห์): URL ที่เข้าชม แต่ละรายการพร้อม
  บัญชีที่บันทึกรายการนั้น เวลาที่เข้าชมครั้งล่าสุด และจำนวนครั้งที่เข้าชม ตัวสำรวจไฟล์ (File Explorer)
  ก็บันทึกไฟล์ในเครื่องที่เปิดไว้ที่นี่ด้วย ในรูปแบบ URL `file:///`
- **การดาวน์โหลด** (`iedownload`): URL ที่ไฟล์ดาวน์โหลดมา ตำแหน่งที่บันทึกไฟล์และตำแหน่งที่เขียนไฟล์
  ระหว่างดาวน์โหลด ขนาด เวลาที่การดาวน์โหลดเริ่มต้น หน้าที่อ้างอิง และชนิด MIME
- **คุกกี้**: ใน Windows 10 ขึ้นไปคือตัวคุกกี้เอง (ตาราง `CookieEntryEx`) พร้อมโฮสต์ ชื่อ ค่า และเวลา
  ส่วนก่อนหน้านั้นคือไฟล์คุกกี้ที่เก็บคุกกี้เหล่านั้นไว้
- **แคช** (`Content`): ไฟล์ที่ดาวน์โหลดมาเพื่อแสดงหน้าเว็บ แต่ละไฟล์พร้อม URL ตำแหน่งของสำเนาบนดิสก์
  และ HTTP response header โปรแกรมอื่นที่ใช้ WinINet ก็แคชสิ่งที่ดาวน์โหลดไว้ที่นี่ด้วย
- เว็บสตอเรจ (`DOMStore`) รายการความเข้ากันได้ (`iecompat`) และอื่น ๆ

ฐานข้อมูลนี้อยู่นานกว่าเบราว์เซอร์: Internet Explorer ถูกยุติการใช้งานแล้ว และ Edge ที่ใช้ Chromium
เป็นพื้นฐานเก็บฐานข้อมูล SQLite ของตนเอง (ดู [เว็บเบราว์เซอร์](browser.md)) แต่ฐานข้อมูลนี้ยังคงอยู่ใน
ทุกโปรไฟล์ พร้อมสิ่งที่ถูกบันทึกไว้ก่อนหน้า และสิ่งที่ตัวสำรวจไฟล์และโปรแกรมอื่นยังคงบันทึกอยู่

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โปรไฟล์ของผู้ใช้บันทึก URL การดาวน์โหลด
หรือไฟล์ที่แคชไว้ ณ เวลาที่บันทึกไว้ ฐานข้อมูลไม่ได้บอกว่าใครเป็นผู้ใช้งานเครื่องอยู่ในขณะนั้น
และไม่ได้บอกว่าไฟล์ที่แคชไว้ถูกเปิด

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Windows\WebCache\WebCacheV01.dat
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01*.log
%LOCALAPPDATA%\Microsoft\Windows\WebCache\V01.chk
```

ไฟล์ที่คอนเทนเนอร์ระบุถึง (ไฟล์ที่แคชไว้ ไฟล์คุกกี้) อยู่ในโฟลเดอร์ของตนเอง ได้แก่
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\`, `INetCookies\`, `History\` และภายใต้
`%LOCALAPPDATA%\Packages\` สำหรับแอป

ให้เก็บรวบรวมโฟลเดอร์ `WebCache` ทั้งหมด Windows เปิดฐานข้อมูลค้างไว้ตลอดเวลาที่ผู้ใช้ลงชื่อเข้าใช้อยู่
สำเนาจากระบบที่กำลังทำงานจึงมักอยู่ในสถานะ dirty โดยการเปลี่ยนแปลงล่าสุดจะมีอยู่เฉพาะในบันทึกธุรกรรม
`V01*.log`

!!! warning "บันทึกธุรกรรม"
    Juntei ไม่เล่นซ้ำบันทึกธุรกรรมของ ESE ฐานข้อมูลที่ไม่ได้ถูกปิดอย่างสมบูรณ์จะถูกนับในสรุปว่าอยู่ในสถานะ
    dirty และถูกรายงานเป็นปัญหา การเปลี่ยนแปลงที่ยังมีอยู่เฉพาะในบันทึกจะขาดหายไป หากต้องการรวม
    การเปลี่ยนแปลงเหล่านั้น ให้เล่นซ้ำบันทึกลงในสำเนาของโฟลเดอร์ก่อน โดยรัน `esentutl /r V01 /d` ของ
    Windows ในสำเนานั้น

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- เวลาเป็น UTC ละเอียดถึงระดับ 100 นาโนวินาที ยกเว้น `ModifiedTime` ของคอนเทนเนอร์ประวัติรายวันหรือ
  รายสัปดาห์ ซึ่งเป็นเวลาท้องถิ่นของระบบที่เขียนไว้โดยไม่มีค่าชดเชยเวลา (offset) และ `-Z` จะคงค่านี้ไว้ตามเดิม
  ให้แปลงค่านี้ด้วยเขตเวลาของระบบต้นทาง ณ ช่วงเวลานั้น ส่วนต่างระหว่างค่านี้กับ `AccessedTime` มักเป็นค่าชดเชย
  ของเขตเวลานั้น แต่ไม่เสมอไป เพราะทั้งสองค่าอาจเป็นขณะเวลาที่ต่างกัน
- รายการประวัติคือ URL ไม่ใช่การเข้าชมแต่ละครั้ง `AccessedTime` คือการเข้าชมครั้งล่าสุด และ
  `AccessCount` คือจำนวนการเข้าชม คอนเทนเนอร์ `History` เก็บหนึ่งรายการต่อหนึ่ง URL ส่วนคอนเทนเนอร์
  รายวันหรือรายสัปดาห์เก็บรายการของช่วงนั้น โดยชื่อของคอนเทนเนอร์ระบุวันแรกของช่วงและวันถัดจากวันสุดท้าย
  (`MSHist012026110120261102`: 1 พฤศจิกายน 2026)
- URL `file:///` ในประวัติคือไฟล์ในเครื่องที่เปิดจากตัวสำรวจไฟล์และโปรแกรมอื่น ไม่ใช่หน้าเว็บ
- `Account` คือบัญชีที่บันทึกรายการนั้น (`Visited: alice@…`) ส่วน `User` คือโฟลเดอร์โปรไฟล์ที่พาธของ
  ฐานข้อมูลอยู่ภายใน ทั้งสองค่าอาจต่างกันได้ เพราะโฟลเดอร์โปรไฟล์ไม่ได้ตั้งชื่อตามบัญชีเสมอไป และฐานข้อมูล
  ที่ถูกคัดลอกไปไว้ที่อื่นจะไม่มีโฟลเดอร์โปรไฟล์อยู่ในพาธ
- `Package` บอกว่ารายการนั้นเป็นของใคร: ของ Edge รุ่นเก่า (`microsoft.microsoftedge_8wekyb3d8bbwe`)
  ของ Internet Explorer ในโหมดป้องกันขั้นสูง (Enhanced Protected Mode) (`windows_ie_ac_001`) หรือของ
  แอปอื่น หากว่างเปล่า รายการนั้นเป็นของผู้ใช้เอง ได้แก่ Internet Explorer ตัวสำรวจไฟล์ และโปรแกรมอื่น
- Microsoft ไม่ได้เผยแพร่เอกสารเกี่ยวกับเรคคอร์ดการดาวน์โหลด Juntei อ่านเวลาเริ่มต้นและขนาดเฉพาะเมื่อ
  ส่วนหัวของเรคคอร์ดอยู่ภายในขอบเขตของเรคคอร์ดนั้น และอ่านพาธกับสตริงอื่นเฉพาะเมื่อสตริงสองตัวสุดท้ายเป็น URL
  และพาธของ Windows มิฉะนั้นแถวจะมี URL ของรายการเอง และเวลาเข้าถึงของรายการเมื่ออ่านเวลาเริ่มต้นไม่ได้
  พร้อมรายงานปัญหา `Publisher` คือชื่อที่เบราว์เซอร์บันทึกไว้ ไม่ใช่ผลการตรวจสอบลายเซ็นของไฟล์
- รายการแคชบอกว่าไฟล์ถูกดาวน์โหลดและแคชไว้ สำเนาบนดิสก์อาจหายไปแล้ว ถูกแทนที่ หรือไม่เคยถูกเปิดเลย
- ยังไม่กู้คืนรายการที่ถูกลบ ESE อาจเก็บเรคคอร์ดที่ถูกลบไว้ในเพจจนกว่าจะล้างเพจเหล่านั้น แต่ Juntei
  อ่านเฉพาะเรคคอร์ดที่ยังใช้งานอยู่

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese webcache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find WebCacheV01.dat
  -f, --file <FILE>      Parse one WebCache database

Output:
  -o, --output <PATH>       Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ese webcache -d Users -o webcache-csv
juntei parse-ese webcache -d Users -t jsonl \
  | jq 'select(.Table == "Downloads") | {StartTime, URL, TargetPath, User}'
```

`-d` ค้นหา `WebCacheV01.dat` ตามชื่อ ส่วน `-f` อ่านฐานข้อมูลไม่ว่าไฟล์จะมีชื่ออะไร

## ตารางเอาต์พุต { #output-tables }

CSV จะสร้างห้าไฟล์ โดยแต่ละไฟล์มีเพียงแถวส่วนหัวเมื่อไม่มีแถวข้อมูล แถว JSON และ JSONL มี `Table`
(`History`, `Downloads`, `Cookies`, `Entries` หรือ `Containers`) ต่อจาก `SourceFile`

| ไฟล์ CSV | เนื้อหา |
|---|---|
| `WebCache-History.csv` | รายการของคอนเทนเนอร์ประวัติ |
| `WebCache-Downloads.csv` | การดาวน์โหลด |
| `WebCache-Cookies.csv` | คุกกี้ (Windows 10 ขึ้นไป) |
| `WebCache-Entries.csv` | รายการของคอนเทนเนอร์อื่นทั้งหมด ได้แก่ ไฟล์ที่แคชไว้ ไฟล์คุกกี้ เว็บสตอเรจ รายการความเข้ากันได้ และอื่น ๆ |
| `WebCache-Containers.csv` | คอนเทนเนอร์ |

## ฟิลด์ { #fields }

ทุกตารางขึ้นต้นด้วยฟิลด์เหล่านี้

| ฟิลด์ | การตีความ |
|---|---|
| `SourceFile` | ฐานข้อมูลที่อ่านแถวนั้นมา |
| `User` | ผู้ใช้ Windows ที่เป็นเจ้าของโปรไฟล์ซึ่งพาธนั้นอยู่ (โฟลเดอร์ถัดจาก `Users`) |
| `Package` | แอปที่เป็นเจ้าของข้อมูลในแถวนั้น: โฟลเดอร์ใต้ `Packages` ในไดเรกทอรีของคอนเทนเนอร์ของแถว (หรือของตารางคุกกี้) ว่างเปล่าสำหรับข้อมูลของผู้ใช้เอง |

### ประวัติ { #history }

| ฟิลด์ | การตีความ |
|---|---|
| `Container` | คอนเทนเนอร์: `History` หรือ `MSHist01` ตามด้วยวันแรกของช่วงและวันถัดจากวันสุดท้าย |
| `AccessedTime` | การเข้าชม URL ครั้งล่าสุด (UTC) ในคอนเทนเนอร์รายวันหรือรายสัปดาห์คือครั้งล่าสุดในช่วงนั้น |
| `ModifiedTime` | เวลาที่รายการถูกเปลี่ยนแปลงครั้งล่าสุด (UTC) ในคอนเทนเนอร์รายวันหรือรายสัปดาห์เป็นเวลาท้องถิ่นของระบบโดยไม่มีค่าชดเชยเวลา |
| `CreationTime` | เวลาที่สร้างรายการ (UTC) |
| `ExpiryTime`, `SyncTime` | เวลาที่รายการมีกำหนดจะถูกนำออกจากประวัติ และเวลาที่ซิงโครไนซ์ครั้งล่าสุด (UTC) |
| `Account` | บัญชีที่บันทึกรายการนั้น |
| `URL` | หน้าเว็บหรือไฟล์ โดยไม่มีคำนำหน้า `Visited: account@` |
| `AccessCount`, `SyncCount` | จำนวนครั้งที่เข้าชม URL นั้น และจำนวนครั้งที่ซิงโครไนซ์รายการ |
| `EntryType`, `Flags` | ชนิดและแฟล็กของรายการตามที่จัดเก็บไว้ |
| `EntryID`, `ContainerID` | id ของรายการในคอนเทนเนอร์ และ id ของคอนเทนเนอร์ |

### การดาวน์โหลด { #downloads }

| ฟิลด์ | การตีความ |
|---|---|
| `StartTime` | เวลาที่การดาวน์โหลดเริ่มต้น (UTC) |
| `URL` | แหล่งที่มาของไฟล์ |
| `TargetPath`, `CurrentPath` | ตำแหน่งที่บันทึกไฟล์ และตำแหน่งที่เขียนไฟล์ระหว่างดาวน์โหลด: ไฟล์ `.partial` สำหรับ Edge รุ่นเก่า และสำเนาในแคชสำหรับ Internet Explorer |
| `TotalBytes` | ขนาดของไฟล์ |
| `Referrer` | หน้าที่อ้างอิง |
| `MimeType` | ชนิดของไฟล์ตามที่เซิร์ฟเวอร์ส่งมา (Internet Explorer) |
| `Publisher` | ชื่อผู้เผยแพร่ไฟล์ที่ Edge รุ่นเก่าบันทึกไว้ |
| `AccessedTime` | เวลาเข้าถึงของรายการเอง (UTC) ซึ่งส่วนใหญ่คือเวลาเริ่มต้น |
| `DownloadID`, `ContainerID` | id ของรายการในคอนเทนเนอร์ และ id ของคอนเทนเนอร์ |

### คุกกี้ { #cookies }

| ฟิลด์ | การตีความ |
|---|---|
| `LastModified`, `Expires` | เวลาที่ตั้งค่าคุกกี้ และเวลาหมดอายุ (UTC) |
| `Host`, `Name`, `Path` | ไซต์ที่คุกกี้นั้นใช้ ชื่อ และพาธของคุกกี้ |
| `Value` | ค่าตามที่จัดเก็บไว้ |
| `Flags` | แฟล็กของคุกกี้ตามที่จัดเก็บไว้ |
| `Directory` | ไดเรกทอรีที่ฐานข้อมูลกำหนดให้กับตารางคุกกี้ ซึ่งบอกว่าคุกกี้เป็นของใคร (`…\INetCookies\ESE\`, `…\MicrosoftEdge\Cookies\ESE\` ของ Edge รุ่นเก่า หรือของแอป) |
| `CookieTable`, `EntryID` | ตาราง (`CookieEntryEx_<n>`) และ id ของคุกกี้ในตารางนั้น |

### รายการ { #entries }

| ฟิลด์ | การตีความ |
|---|---|
| `Container` | คอนเทนเนอร์: `Content` (ไฟล์ที่แคชไว้), `Cookies` (ไฟล์คุกกี้), `DOMStore` (เว็บสตอเรจ), `iecompat`, … |
| `AccessedTime` | เวลาที่ใช้รายการนั้นครั้งล่าสุด (UTC) |
| `ModifiedTime` | เวลาที่ทรัพยากรถูกแก้ไขครั้งล่าสุดตามที่เซิร์ฟเวอร์ระบุ (UTC) |
| `CreationTime` | เวลาที่สร้างรายการ (UTC) |
| `ExpiryTime`, `SyncTime` | เวลาหมดอายุ และเวลาที่ตรวจสอบกับเซิร์ฟเวอร์ครั้งล่าสุด (UTC) |
| `URL` | URL ตามที่จัดเก็บไว้: URL สำหรับไฟล์ที่แคชไว้, `Cookie:alice@example.com/` สำหรับไฟล์คุกกี้, `DOMStore:…`, `iecompat:…` |
| `Account` | บัญชีของไฟล์คุกกี้ |
| `Filename`, `FilePath` | ไฟล์ของรายการ และตำแหน่งของไฟล์: ไดเรกทอรีของคอนเทนเนอร์และโฟลเดอร์ย่อยที่รายการระบุ |
| `FileSize` | ขนาดของไฟล์ |
| `AccessCount`, `SyncCount` | จำนวนครั้งที่ใช้รายการนั้น และจำนวนครั้งที่ตรวจสอบกับเซิร์ฟเวอร์ |
| `EntryType`, `Flags` | ชนิดและแฟล็กของรายการตามที่จัดเก็บไว้ |
| `RedirectURL` | ปลายทางที่ URL เปลี่ยนเส้นทางไป |
| `ResponseHeaders`, `RequestHeaders` | HTTP response header (`HTTP/1.1 200 OK`, `Content-Type`, `Server`, …) และ request header ตามที่จัดเก็บไว้ ครบทุกบรรทัด |
| `EntryID`, `ContainerID` | id ของรายการในคอนเทนเนอร์ และ id ของคอนเทนเนอร์ |

### คอนเทนเนอร์ { #containers }

| ฟิลด์ | การตีความ |
|---|---|
| `ContainerID`, `Name` | id และชื่อของคอนเทนเนอร์ |
| `Directory` | ตำแหน่งไฟล์ของคอนเทนเนอร์บนดิสก์ |
| `SecureDirectories` | โฟลเดอร์ย่อยของ `Directory` ที่เก็บไฟล์ของคอนเทนเนอร์ (ใน CSV คั่นด้วยจุลภาค) |
| `PartitionID` | `M` สำหรับคอนเทนเนอร์ของผู้ใช้เอง `L` สำหรับคอนเทนเนอร์ของโหมดป้องกัน (Protected Mode) ของ Internet Explorer (โฟลเดอร์ `Low`) และ SID ของ app container สำหรับคอนเทนเนอร์ของแอป |
| `LastAccessTime`, `LastScavengeTime` | เวลาที่ใช้คอนเทนเนอร์ครั้งล่าสุด และเวลาที่ล้างข้อมูลครั้งล่าสุด (UTC) |

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียนตารางลงใน `BrowserActivity` เป็น `WebCache-History.csv`,
`WebCache-Downloads.csv`, `WebCache-Cookies.csv`, `WebCache-Entries.csv` และ
`WebCache-Containers.csv` (เป็นไฟล์ `WebCache.json` หรือ `WebCache.jsonl` ไฟล์เดียวเมื่อใช้ `-t json`
หรือ `-t jsonl`) พร้อม `BrowserActivity/WebCache-logs.txt` ไทม์ไลน์แบบรวม `-j` จะรวมทุกแถว พร้อม `Comp`
และตัวระบุ `Artifact` ที่เป็น `webcache` ส่วน `-Z` จะแปลงเวลาทั้งหมด ได้แก่ ฟิลด์ที่ลงท้ายด้วย `Time`,
`LastModified` และ `Expires` ยกเว้น `ModifiedTime` ที่เป็นเวลาท้องถิ่นของคอนเทนเนอร์ประวัติรายวันหรือรายสัปดาห์

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft — About WinINet](https://learn.microsoft.com/en-us/windows/win32/wininet/about-wininet)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — ESE database file format](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [Plaso — MSIE WebCache parser](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/esedb_plugins/msie_webcache.py)
- [dissect.database](https://github.com/fox-it/dissect.database)
