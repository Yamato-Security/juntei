# ดัชนี Windows Search { #windows-search-index }

## สิ่งที่ดัชนีบันทึกไว้ { #what-the-index-records }

Windows Search (บริการ `WSearch`) จะรวบรวมข้อมูล (crawl) จากโปรไฟล์ผู้ใช้ เมนู Start กล่องจดหมายของ Outlook ประวัติของ Internet Explorer ไฟล์ออฟไลน์ และสิ่งอื่นใดที่อยู่ในขอบเขตของมัน แล้วเก็บสิ่งที่อ่านได้ไว้ใน `Windows.edb` ซึ่งเป็นฐานข้อมูล ESE สองส่วนของฐานข้อมูลนี้มีความสำคัญต่อการตรวจสอบ:

- **property store** (`SystemIndex_PropertyStore` หรือ `SystemIndex_0A` บน Windows 7) ซึ่งมีหนึ่งแถวต่อรายการที่ทำดัชนีไว้ ได้แก่ พาธ ชื่อ ชนิด และขนาด เจ้าของ เวลาสร้าง แก้ไข และเข้าถึงที่ตัวสร้างดัชนี (indexer) คัดลอกมาจากไฟล์ เวลาที่ตัวสร้างดัชนีอ่านรายการนั้น ชื่อเรื่องและผู้เขียนของเอกสาร ส่วนต้นของข้อความ ชื่อคอมพิวเตอร์ และคุณสมบัติ (property) อื่นอีกหลายร้อยรายการ
- ตาราง **gatherer** (`SystemIndex_Gthr`, `SystemIndex_GthrPth`) ซึ่งเป็นระเบียนของ crawler เองสำหรับแต่ละรายการ ได้แก่ พาธแบบ URL เวลาแก้ไขขณะที่ถูกจัดเข้าคิว หมายเลขและแฟล็กของการรวบรวม (crawl) และตัวนับการลบ

รายการจะยังคงอยู่ในดัชนีหลังจากไฟล์ ข้อความ หรือหน้าเว็บหายไปแล้ว จนกว่า crawler จะสังเกตเห็นการลบ ดัชนีจึงระบุชื่อไฟล์ที่ไม่มีอยู่แล้ว พร้อมเมทาดาทาและข้อความที่คัดมาบางส่วน ตามสภาพ ณ เวลาที่ทำดัชนี

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ตัวสร้างดัชนีได้อ่านรายการที่พาธที่บันทึกไว้ พร้อมเมทาดาทาที่บันทึกไว้ ณ เวลา `GatherTime` บนคอมพิวเตอร์ที่ระบุชื่อ ดัชนีแสดงว่าไฟล์มีอยู่และมีเนื้อหาอะไร แต่ไม่ได้แสดงว่าผู้ใช้เปิดไฟล์นั้น

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\ProgramData\Microsoft\Search\Data\Applications\Windows\Windows.edb
```

Windows 7 ถึง Windows 11 21H2 และ Windows Server รุ่นที่ออกในปีเดียวกัน เก็บดัชนีไว้ใน `Windows.edb` ส่วน Windows 11 22H2 ขึ้นไปเก็บไว้ในฐานข้อมูล SQLite ในโฟลเดอร์เดียวกัน (`Windows.db`, `Windows-gather.db`, `Windows-usn.db`) Juntei จะค้นหาและระบุชื่อไฟล์เหล่านี้ และรายงานว่าเป็นรูปแบบใหม่กว่าที่ยังไม่รองรับ

ให้เก็บรวบรวมโฟลเดอร์ `Applications\Windows` ทั้งหมดจากอิมเมจดิสก์ Volume Shadow Copy หรือหลังจากหยุดบริการ `WSearch` แล้ว เพราะฐานข้อมูลถูกเปิดค้างไว้ขณะที่บริการทำงาน และการเปลี่ยนแปลงล่าสุดจะมีอยู่เฉพาะในบันทึกธุรกรรม `MSS*.log` ซึ่ง Juntei ไม่ได้เล่นซ้ำ ดัชนีที่คัดลอกจากระบบที่กำลังทำงานจะอยู่ในสถานะ dirty เอาต์พุตจะระบุเช่นนั้น และอ่านสิ่งที่อยู่บนดิสก์ อินพุตแบบไดเรกทอรีและ `auto-parse` จะอ่าน `Windows.edb` ทุกไฟล์ไม่ว่าจะอยู่ลึกเพียงใด ใช้ `-f` สำหรับไฟล์เดียว

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- วิเคราะห์สำเนาหลักฐานที่ไม่มีการเปลี่ยนแปลง Juntei ใช้การแมปไฟล์เข้าหน่วยความจำเพื่อให้อ่านได้อย่างมีประสิทธิภาพ ห้ามแก้ไขหรือตัดทอนฐานข้อมูลระหว่างอ่าน

- `GatherTime` คือเวลาที่ตัวสร้างดัชนีอ่านรายการ ไม่ใช่เวลาที่ผู้ใช้สร้าง เปลี่ยนแปลง หรือเปิดรายการนั้น การทำดัชนีทำงานอยู่เบื้องหลัง มักเกิดขึ้นหลังการเปลี่ยนแปลงหลายนาทีถึงหลายชั่วโมง และจะหยุดชั่วคราวขณะที่คอมพิวเตอร์กำลังทำงานหนักหรือใช้แบตเตอรี่
- `DateCreated`, `DateModified` และ `DateAccessed` เป็นสำเนาที่ตัวสร้างดัชนีคัดลอกมาจากไฟล์ ค่าเหล่านี้จะถูกปรับปรุงใหม่เมื่อรายการถูกทำดัชนีอีกครั้ง จึงอธิบายไฟล์ ณ เวลา `GatherTime`
- `ItemPath` คือพาธสำหรับแสดงผล ส่วน `ItemURL` และ `GatherPath` คือ URL ของ crawler ได้แก่ `file:` สำหรับไฟล์ (โดยวอลุ่มอยู่ในรูป `[GUID]` บน Windows 8 ขึ้นไป), `iehistory://{SID}/` สำหรับประวัติของ Internet Explorer, `mapi://` สำหรับรายการของ Outlook, `csc://{SID}/` สำหรับไฟล์ออฟไลน์ และ `winrt://{SID}/` สำหรับเนื้อหาของแอป Windows
- `User` คือโปรไฟล์ที่ระบุชื่อในพาธของรายการ (`C:\Users\<name>\…`) เป็นตำแหน่งที่รายการอยู่ ไม่ใช่ผู้ที่สร้างรายการ ส่วน `FileOwner` คือเจ้าของไฟล์ตามที่ตัวสร้างดัชนีเห็น
- `AutoSummary` คือส่วนต้นของข้อความในรายการ อาจอ้างถึงเอกสารที่ถูกลบหรือเข้ารหัสไปแล้วในภายหลัง และเป็นข้อความที่คัดมาแบบถูกตัดสั้น ไม่ใช่ตัวเอกสาร
- ฐานข้อมูลที่อยู่ในสถานะ dirty อาจมีเพจที่เพจกิ่ง (branch page) ยังไปไม่ถึง Juntei จะไล่ตามสายโซ่ของเพจใบ (leaf page) เอง และระบุในสรุปว่าอ่านได้กี่เพจด้วยวิธีนี้ การเปลี่ยนแปลงล่าสุดของฐานข้อมูลยังคงอยู่ในบันทึกธุรกรรม
- รายการในตาราง gatherer ที่ไม่มีแถวใน property store (`InTables` เป็น `Gthr`) คือรายการที่ถูกจัดเข้าคิวหรือถูกรวบรวมแล้ว แต่คุณสมบัติยังไม่ถูกเขียนหรือถูกลบออกไปแล้ว พาธ ชื่อ และเวลาของ gatherer ยังคงอยู่
- Windows 7 เก็บสตริงในการเข้ารหัส (encoding) แบบบีบอัดและปิดบังของตนเอง Juntei ถอดรหัสค่าเหล่านี้ ค่าที่ถอดรหัสไม่ได้จะคงอยู่ในรูปเลขฐานสิบหก พร้อมหมายเหตุใน `Problems`

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ese windows-search <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Directory to search recursively for Windows Search indexes
                         (Windows.edb; Windows.db is reported as the unsupported SQLite index)
  -f, --file <FILE>      File path to one Windows.edb

Output:
  -o, --output <FILE>       Save every indexed item to this file
  -t, --output-type <TYPE>  csv (default), json, or jsonl
  -C, --clobber             Overwrite the output file when saving
```

```bash
juntei parse-ese windows-search -f Windows.edb -o windows-search.csv
juntei parse-ese windows-search -d evidence/ -t jsonl \
  | jq 'select(.Store == "file") | {ItemPath, DateModified, GatherTime, FileOwner, AutoSummary}'
```

CSV เป็นรูปแบบไฟล์เริ่มต้น เมื่อใช้ `-t csv`, `-t json` หรือ `-t jsonl` โดยไม่มี `-o` จะเขียนระเบียนไปยัง stdout หากไม่มีทั้งสองตัวเลือกจะแสดงเพียงสรุป เพิ่ม `-C` เพื่อเขียนทับเอาต์พุตที่มีอยู่ เอาต์พุตจะไม่ถูกเขียนทับไฟล์อินพุตใด ๆ เลย แม้จะใช้ `-C`

Juntei เขียนหนึ่งระเบียนต่อรายการที่ทำดัชนีไว้หนึ่งรายการ โดยเชื่อม (join) property store กับตาราง gatherer ด้วยตัวระบุของรายการ สรุปจะนับฐานข้อมูลตามสคีมา ฐานข้อมูลที่อยู่ในสถานะ dirty รายการที่อยู่ในทั้งสองตาราง รายการที่อยู่เฉพาะใน gatherer และเฉพาะใน property store โฟลเดอร์ รายการที่ถูกทำเครื่องหมายว่าลบแล้ว รายการต่อ store รายการที่ไม่มีเวลาที่จัดเก็บไว้ และปัญหา

ความเสียหายไม่ทำให้การทำงานหยุดลง ฐานข้อมูลที่ถูกตัดสั้นจะเก็บรายการที่เพจที่เหลืออยู่มีไว้ และสรุปจะบอกว่าอ่านอะไรไม่ได้ ไฟล์ชื่อ `Windows.edb` ที่ไม่ใช่ดัชนี Windows Search จะถูกรายงานและข้ามไป ฐานข้อมูลอื่นยังคงถูกอ่านตามปกติ

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `DocumentID` | ตัวระบุของรายการในดัชนี ได้แก่ `WorkID` ของ property store (`DocID` บน Windows 7) และ `DocumentID` ของแถว gatherer |
| `InTables` | `Gthr+PropertyStore`, `Gthr` (รายการที่ถูกจัดเข้าคิวหรือรวบรวมแล้วแต่ไม่มีคุณสมบัติ) หรือ `PropertyStore` |
| `ItemPath` | `System_ItemPathDisplay` คือพาธตามที่ Windows แสดง ว่างเปล่าเมื่อ property store ไม่มีแถวของรายการนั้น |
| `ItemURL` | `System_ItemUrl` คือ URL ของ crawler สำหรับรายการ (`file:`, `iehistory://`, `mapi://`, `csc://`, `winrt://`) |
| `GatherPath` | พาธของ gatherer ซึ่งประกอบขึ้นจากต้นไม้ขอบเขต (scope) และชื่อไฟล์ของแถว |
| `Name` | `System_ItemName` หรือชื่อไฟล์ของแถว gatherer |
| `ItemType`, `ItemTypeText` | `System_ItemType` (นามสกุลไฟล์ หรือ `Directory`) และชื่อสำหรับแสดงผล |
| `Kind` | `System_Kind` ได้แก่ `document`, `folder`, `link`, `program`, `email`, `picture`, … หากมีหลายค่าจะเชื่อมด้วย `\|` |
| `MimeType` | `System_MIMEType` |
| `Store` | `System_Search_Store` ได้แก่ `file`, `mapi` (Outlook), `iehistory`, `csc` (ไฟล์ออฟไลน์) หรือ `winrt` (แอป Windows) |
| `IsFolder`, `IsDeleted` | `System_IsFolder` และ `System_IsDeleted` ตามที่จัดเก็บไว้ |
| `DeletedCount` | ตัวนับการลบของแถว gatherer |
| `Size` | `System_Size` เป็นไบต์ ว่างเปล่าสำหรับโฟลเดอร์ ซึ่งเก็บค่าตัวแทน (placeholder) ไว้ |
| `FileAttributes` | `System_FileAttributes` เป็นตัวเลข (16 ไดเรกทอรี, 32 อาร์ไคฟ์, 2 ซ่อน, …) |
| `FileOwner` | `System_FileOwner` (`DOMAIN\user`) ตามที่ตัวสร้างดัชนีอ่านได้ |
| `DateCreated`, `DateModified`, `DateAccessed` | เวลาของไฟล์ตามที่ตัวสร้างดัชนีคัดลอกมา (UTC) |
| `ItemDate` | `System_ItemDate` คือวันที่ของตัวรายการเอง เช่น เวลาส่งของข้อความ หรือวันที่ถ่ายของรูปภาพ สำหรับไฟล์มักเป็นเวลาแก้ไขละเอียดถึงระดับวินาที |
| `GatherTime` | `System_Search_GatherTime` คือเวลาที่ตัวสร้างดัชนีอ่านรายการ (UTC) |
| `GatherLastModified` | `LastModified` ของแถว gatherer คือเวลาแก้ไขของรายการขณะที่ crawler จัดเข้าคิว (UTC) |
| `Title`, `Author` | `System_Title` และ `System_Author` |
| `AutoSummary` | `System_Search_AutoSummary` คือส่วนต้นของข้อความในรายการ |
| `CompName` | `System_ComputerName` คือคอมพิวเตอร์ที่เป็นเจ้าของดัชนี |
| `TargetURL`, `DateVisited` | URL และเวลาเข้าชมของรายการประวัติ Internet Explorer (`System_Link_TargetUrl`, `System_Link_DateVisited`) |
| `ActivityApp`, `ActivityText`, `ActivityContentURI`, `ActivityStartTime`, `ActivityEndTime` | รายการประวัติกิจกรรมของ Windows 10 (`System_Activity_*`, `System_ActivityHistory_*`) |
| `User`, `UserSource` | โปรไฟล์ที่ระบุชื่อในพาธของรายการ และพาธที่อ่านค่านั้นมา (`ItemPath`, `ItemURL` หรือ `GatherPath`) ว่างเปล่าเมื่ออยู่นอกโปรไฟล์ |
| `ScopeID`, `SDID` | ตัวระบุขอบเขต (scope) และตัวบอกความปลอดภัยของแถว gatherer |
| `CrawlNumber` | `CrawlNumberCrawled` ของแถว gatherer |
| `GatherFlags` | `TransactionFlags` ของแถว gatherer |
| `Priority` | ลำดับความสำคัญของการรวบรวมสำหรับรายการที่ยังอยู่ในคิว ESE จะทำเครื่องหมายเป็น NULL เมื่อรายการถูกรวบรวมแล้ว จึงว่างเปล่าสำหรับรายการส่วนใหญ่ |
| `PropertyCount` | จำนวนคุณสมบัติที่ property store เก็บไว้สำหรับรายการนั้น |
| `OtherProperties` | คุณสมบัติอื่นทั้งหมด ในรูป `Name=Value` คั่นด้วย `; ` |
| `SourceFile` | ฐานข้อมูลที่อ่าน เป็นพาธสัมพัทธ์จากไดเรกทอรีอินพุต หรือชื่อไฟล์เมื่อใช้ `-f` |
| `Problems` | ค่าที่ถอดรหัสไม่ได้ เวลาที่ไม่ใช่เวลา แถวซ้ำ และความเสียหาย ห้ามซ่อนไว้ |

JSON และ JSONL มีฟิลด์เดียวกันโดยค่ามีชนิดข้อมูล ตัวเลขยังคงเป็นตัวเลข และค่าที่ขาดหายเป็น `null` แทนที่จะมี `OtherProperties` จะมี `Properties` ซึ่งเป็นคุณสมบัติทุกรายการของรายการนั้นพร้อมชื่อ ตัวระบุที่เป็นตัวเลข ชนิดคอลัมน์ และค่า และสำหรับเวลาจะมีจำนวน tick ดิบของ FILETIME ไว้ข้างเวลาที่อ่านได้ `Gather` เก็บคอลัมน์ของแถว gatherer

## พฤติกรรมในไทม์ไลน์ { #timeline-behavior }

`auto-parse` บันทึกระเบียนเป็น `FileExistence/WindowsSearch.csv` (หรือ `.json`/`.jsonl`) และเพิ่มลงในไทม์ไลน์แบบรวม `-j` ส่วน `super-timeline` สร้างเหตุการณ์จาก `DateCreated`, `DateModified` และ `DateAccessed` (เวลาของไฟล์ตามที่ Windows Search บันทึกไว้) จาก `GatherTime` (เวลาที่รายการถูกทำดัชนี) จาก `DateVisited` ของรายการประวัติ Internet Explorer และจาก `ActivityStartTime` และ `ActivityEndTime` ของกิจกรรม `GatherLastModified` จะกลายเป็นเหตุการณ์เฉพาะสำหรับรายการที่ property store ไม่มีเวลาแก้ไขให้ รายการที่ไม่มีเวลาใดเลยในกลุ่มนี้จะยังอยู่ในเอาต์พุตที่วิเคราะห์แล้ว และนับเป็นรายการที่ไม่มีเวลา

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. กรอง `Store` ให้เป็น `file` แล้วค้นหาไฟล์ ชื่อ และคำที่เกี่ยวข้องกับคดีใน `ItemPath`, `Name` และ `AutoSummary` ดัชนีระบุชื่อไฟล์ที่ถูกลบไปแล้วในภายหลังด้วย
2. อ่าน `GatherTime` เป็นเวลาล่าสุดที่ไฟล์มีอยู่พร้อมเมทาดาทาที่บันทึกไว้ และอ่าน `DateModified` เป็นเวลาแก้ไขของไฟล์ ณ ขณะนั้น
3. ใช้ `FileOwner` และ `User` เพื่อเชื่อมโยงไฟล์กับบัญชี แล้วยืนยันด้วย `$MFT`, USN Journal และ `$Secure:$SDS`
4. สำหรับอีเมล (`Store` เป็น `mapi`) และประวัติ Internet Explorer (`iehistory`) ให้เชื่อมโยงต่อไปยังไฟล์ของ Outlook และ WebCache เพื่อดูระเบียนฉบับเต็ม
5. เปรียบเทียบดัชนีระหว่างโฮสต์ `CompName` ระบุชื่อคอมพิวเตอร์ที่เป็นเจ้าของดัชนี และดัชนีที่เก็บรวบรวมจากอิมเมจจะระบุชื่อไฟล์บนอิมเมจนั้น

## เวอร์ชันที่ทดสอบ { #tested-versions }

การทดสอบใช้ดัชนีสาธารณะที่มีผลลัพธ์ซึ่งเครื่องมืออื่นอ่านได้อย่างอิสระ:

- ดัชนี Windows 7 ของ plaso (`SystemIndex_0A`) ตรวจสอบเทียบกับ `esedbexport` ของ libyal รวมถึงการเข้ารหัสสตริงของ Windows 7
- ดัชนี Windows 10 ของ DFIRArtifactMuseum ซึ่งปล่อยไว้ในสถานะ dirty ตรวจสอบเทียบกับ SIDR ของ Stroz Friedberg และดัชนี Windows 11 ของแหล่งเดียวกัน (รูปแบบ ESE รุ่นแก้ไข 200) ตรวจสอบเทียบกับ `esedbexport`
- ดัชนี SQLite ของ Windows 11 22H2 ของ DFIRArtifactMuseum ซึ่งรายงานว่าเป็นรูปแบบใหม่กว่าที่ยังไม่รองรับ
- ฐานข้อมูลที่ถูกตัดสั้น ฐานข้อมูลว่าง หรือฐานข้อมูลชนิดอื่น

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft — ภาพรวม Windows Search](https://learn.microsoft.com/en-us/windows/win32/search/-search-3x-wds-overview)
- [Microsoft — Extensible Storage Engine](https://learn.microsoft.com/en-us/windows/win32/extensible-storage-engine/extensible-storage-engine)
- [libesedb — รูปแบบไฟล์ฐานข้อมูล ESE](https://github.com/libyal/libesedb/blob/main/documentation/Extensible%20Storage%20Engine%20%28ESE%29%20Database%20File%20%28EDB%29%20format.asciidoc)
- [esedb-kb — Windows Search](https://github.com/libyal/esedb-kb/blob/main/documentation/Windows%20Search.asciidoc)
- [Stroz Friedberg — SIDR](https://github.com/strozfriedberg/sidr)
- [Microsoft — ซอร์สโค้ด Extensible Storage Engine](https://github.com/microsoft/Extensible-Storage-Engine)
