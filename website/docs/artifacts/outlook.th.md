# Outlook { #outlook }

## ไฟล์ของ Outlook คืออะไร { #what-outlooks-files-are }

Outlook for Windows เก็บอีเมล ผู้ติดต่อ ปฏิทิน และงานไว้ในไฟล์ของตนเอง ได้แก่

- **OST** (`.ost`): สำเนาออฟไลน์ของกล่องจดหมาย ซึ่ง Outlook เก็บไว้สำหรับบัญชี Exchange และ Microsoft 365
  และตั้งแต่ Outlook 2013 ก็เก็บไว้สำหรับบัญชี IMAP ด้วย OST ของ Outlook 2013 ขึ้นไปมีเพจขนาด 4 KiB
  และบล็อกที่ถูกบีบอัด
- **PST** (`.pst`): ที่เก็บข้อมูลส่วนบุคคล ได้แก่ บัญชี POP3 ไฟล์เก็บถาวร (`archive.pst`) ไฟล์ที่ส่งออก
  และที่เก็บข้อมูลที่ผู้ใช้สร้างขึ้น PST แบบ ANSI เป็นของ Outlook 97 ถึง 2002 ส่วน PST แบบ Unicode เป็นของ
  Outlook 2003 ขึ้นไป
- **MSG** (`.msg`): รายการหนึ่งรายการที่บันทึกเป็นไฟล์ เมื่อถูกลากออกจาก Outlook ถูกบันทึกด้วย **Save As**
  หรือถูกส่งออกโดยโปรแกรมอื่น ข้อความที่ถูกแนบมาก็เป็นไฟล์ MSG เช่นกัน
- **OFT** (`.oft`): เทมเพลตของ Outlook ซึ่งเป็นไฟล์ MSG ที่ใช้สร้างรายการใหม่

ที่เก็บข้อมูลแต่ละแห่งมีโฟลเดอร์ และแต่ละโฟลเดอร์มีรายการของตน ได้แก่ อีเมล คำขอประชุมและการตอบกลับ
ผู้ติดต่อและรายชื่อการแจกจ่าย (distribution list) รายการปฏิทิน งาน บันทึกย่อ รายการบันทึกประจำวัน (journal)
และรายงาน แต่ละโฟลเดอร์ยังเก็บรายการที่ซ่อนอยู่ ซึ่งเป็นข้อมูลที่เชื่อมโยงกับโฟลเดอร์ (folder associated
information หรือ FAI) ได้แก่ มุมมอง กฎ ฟอร์ม และการตั้งค่าของ Outlook (รายการ `IPM.Configuration.*` เช่น
รายการ autocomplete ของกล่องจดหมาย Exchange) สิ่งที่แนบของรายการอาจเป็นไฟล์ การอ้างอิงถึงไฟล์ ออบเจกต์ OLE
หรือรายการอื่น (ข้อความที่ถูกส่งต่อเป็นสิ่งที่แนบ หรือข้อยกเว้นของการนัดหมายแบบเกิดซ้ำ) ซึ่งมีสิ่งที่แนบของตนเอง
ได้อีก

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ณ เวลาที่เก็บรวบรวม ที่เก็บข้อมูลหรือไฟล์มีรายการนั้นอยู่
พร้อมผู้ส่ง ผู้รับ เวลา และสิ่งที่แนบเหล่านี้ แต่ไม่ได้บอกว่าผู้ใช้อ่าน เขียน หรือส่งรายการนั้น และไม่ได้บอกว่า
ผู้ส่งเป็นบุคคลตามที่รายการระบุจริง

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Outlook\*.ost
%USERPROFILE%\Documents\Outlook Files\*.pst      (Outlook 2010 and later)
%LOCALAPPDATA%\Microsoft\Outlook\*.pst           (Outlook 2007 and earlier)
%APPDATA%\Microsoft\Templates\*.oft
*.msg wherever users and programs save them: Desktop, Documents, Downloads, …
```

PST อาจอยู่ที่ใดก็ได้ เพราะผู้ใช้ย้ายไฟล์เก็บถาวรไปไว้ในโฟลเดอร์ ไดรฟ์ และการแชร์ไฟล์อื่น Outlook เก็บพาธของ
ที่เก็บข้อมูลของแต่ละโปรไฟล์ไว้ในรีจิสทรีของผู้ใช้ สิ่งที่แนบที่ผู้ใช้เปิดจาก Outlook จะถูกคัดลอกไปยังโฟลเดอร์หนึ่งใน
`%LOCALAPPDATA%\Microsoft\Windows\INetCache\Content.Outlook\` (`Temporary Internet Files\Content.Outlook`
ก่อน Windows 8.1) และอาจยังคงอยู่ที่นั่น

Outlook เปิดที่เก็บข้อมูลค้างไว้ตลอดเวลาที่ทำงาน และ OST หนึ่งไฟล์อาจมีขนาดหลายสิบกิกะไบต์ ให้เก็บรวบรวม
ไฟล์เหล่านี้ขณะที่ Outlook ปิดอยู่ หรือเก็บจากอิมเมจดิสก์หรือ Volume Shadow Copy Juntei อ่านที่เก็บข้อมูลแต่ละแห่ง
แบบสตรีม และไม่เคยโหลดที่เก็บข้อมูลทั้งหมดไว้ในหน่วยความจำ

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- Juntei อ่านแบบผ่อนปรน คือข้ามส่วนที่เสียหาย อ่านส่วนที่อ่านได้ และรายงานส่วนที่เหลือเป็นปัญหา ปัญหาเหล่านี้
  จะถูกพิมพ์ออกมา (10 รายการแรกของแต่ละไฟล์) ถูกนับในสรุปและในตาราง Files และถูกระบุในฟิลด์ `Problems`
  ของรายการและสิ่งที่แนบที่เกี่ยวข้อง รายการที่อ่านไม่ได้เลยก็ยังมีแถวในตาราง Items พร้อม `ItemID` และสาเหตุ
- เวลาเป็น UTC ละเอียดถึงระดับ 100 นาโนวินาที `SubmitTime` คือเวลาที่ไคลเอนต์ของผู้ส่งส่งข้อความออกไป
  ตามนาฬิกาของคอมพิวเตอร์เครื่องนั้น ส่วน `DeliveryTime` คือเวลาที่ข้อความมาถึงกล่องจดหมายนี้ `CreationTime`
  คือเวลาที่สำเนาของรายการนี้ถูกสร้างขึ้น ได้แก่ เวลาที่ถูกส่งถึงหรือถูกสร้างในกล่องจดหมาย ถูกซิงโครไนซ์ลงใน OST
  ถูกคัดลอกหรือนำเข้าสู่ PST หรือถูกบันทึกเป็นไฟล์ MSG ซึ่งไม่ใช่เวลาที่เขียนข้อความ `LastModifiedTime`
  คือเวลาที่รายการถูกเปลี่ยนแปลงครั้งล่าสุดในที่เก็บข้อมูล
- `StartTime` และ `DueTime` ของงานเป็นวันที่ตามเขตเวลาของผู้ใช้ ซึ่ง Outlook จัดเก็บเป็นเวลาเที่ยงคืน Juntei
  เขียนค่าเหล่านี้โดยไม่มีค่าชดเชยเวลา (`2017-02-19T00:00:00`) และ `-Z` จะคงค่าเหล่านี้ไว้ตามเดิม
- Outlook เขียนเที่ยงคืนของวันที่ 1 มกราคม 4501 แทน **None** ในวันที่บางรายการ เช่น วันครบกำหนดของงาน
  หรือเวลาของการแจ้งเตือน Juntei เขียนค่านี้ตามที่จัดเก็บไว้ คือ `4501-01-01T00:00:00Z` และไม่มี `Z` ใน
  `StartTime` และ `DueTime` ของงาน
- `Read` คือแฟล็กการอ่าน ณ เวลาที่เก็บรวบรวมที่เก็บข้อมูล ซึ่งหมายความว่ามีบุคคลหรือกฎทำเครื่องหมายว่ารายการนั้น
  อ่านแล้ว แต่ไม่ได้บอกว่าใครหรือเมื่อใด
- ชื่อและที่อยู่ของผู้ส่งและผู้รับคือสิ่งที่โปรแกรมที่ใช้ส่งและเมลเซิร์ฟเวอร์บันทึกไว้ และอาจถูกปลอมแปลงได้ ที่อยู่
  `EX` คือ distinguished name ของ Exchange (`/o=…/ou=…/cn=Recipients/cn=…`) ส่วนที่อยู่ SMTP จะอยู่ใน
  `SenderSMTP-Address` และ `SMTP-Address` เมื่อรายการจัดเก็บไว้ด้วย หากต้องการตรวจสอบว่าข้อความมาจากที่ใด
  ให้อ่าน Internet header ของข้อความ ได้แก่ `0x007D001F` หรือ `0x007D001E` (`PidTagTransportMessageHeaders`)
  ในตาราง Properties
- รายการที่ถูกลบใน Outlook จะอยู่ใน **Deleted Items** จนกว่าจะล้างโฟลเดอร์นั้น และจะถูกอ่านเหมือนรายการอื่น
  ส่วนรายการที่ไม่อยู่ในตารางของโฟลเดอร์ใดแล้ว ซึ่งที่เก็บข้อมูลอาจยังเก็บไว้จนกว่าจะนำพื้นที่นั้นกลับมาใช้ใหม่
  ยังไม่ถูกอ่าน
- `ItemID` คือ node id ของข้อความใน PST หรือ OST ที่ข้อความนั้นอยู่ รายการเดียวกันในสำเนาอื่นของที่เก็บข้อมูล
  อาจมี id ต่างกัน id ของรายการที่ฝังอยู่คือ id ของรายการแม่ตามด้วยดัชนีของสิ่งที่แนบ (`2097348/0`, `0/0/1`)
  ส่วนรายการของไฟล์ `.msg` หรือ `.oft` มี id เป็น `0`
- `User` มาจากพาธเท่านั้น (โฟลเดอร์ถัดจาก `Users`) และจะหายไปเมื่อคัดลอกไฟล์ไปไว้ที่อื่น PST อาจเป็นของ
  ผู้ใช้คนใดก็ได้
- สตริงที่ไม่ใช่ Unicode จะถูกอ่านด้วย code page ของรายการ (code page ของรายการเอง ของ locale ของรายการ
  หรือ Internet code page ของรายการ) `--code-page` กำหนด code page ให้กับรายการที่ไม่ได้ระบุไว้ ซึ่งควรเป็น
  ANSI code page ของระบบที่เขียนรายการเหล่านั้น เช่น `932` สำหรับระบบภาษาญี่ปุ่น
- รหัสผ่านของ PST ไม่ได้เข้ารหัสไฟล์ Juntei อ่าน PST ที่ตั้งรหัสผ่านไว้ได้เหมือน PST อื่น `Encryption` ของ
  ที่เก็บข้อมูล (`Permute` ซึ่งคือ "compressible encryption" ของ Outlook หรือ `Cyclic`) เพียงทำให้บล็อก
  อ่านได้ยากขึ้นเท่านั้น ข้อความที่เข้ารหัสด้วย S/MIME เก็บเนื้อหาและสิ่งที่แนบไว้ในสิ่งที่แนบที่ถูกเข้ารหัส
  (`smime.p7m`) ซึ่ง Juntei ไม่ถอดรหัส
- สิ่งที่แนบมักเป็นมัลแวร์ `--extract-attachments` เขียนข้อมูลของสิ่งที่แนบลงในไฟล์ที่ตั้งชื่อตาม SHA-256
  โดยไม่มีสิทธิ์ในการเรียกใช้ (execute) ห้ามเรียกใช้ไฟล์เหล่านี้เด็ดขาด และให้เก็บไว้ในระบบวิเคราะห์ที่แยกไว้
  ต่างหาก (isolated)

### สิ่งที่ยังไม่อ่าน { #not-read-yet }

สมุดรายชื่อแบบออฟไลน์ (OAB) สตรีม autocomplete (`Stream_Autocomplete_*.dat`) และไฟล์ `.nk2` การตั้งค่า
การส่ง/รับ (`.srs`) แคชการตั้งค่าของ Outlook (RoamCache, Autodiscover) ฐานข้อมูล SQLite ของ Outlook
และข้อความที่ไม่อยู่ในตารางของโฟลเดอร์ใดแล้ว

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file outlook <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find *.pst, *.ost, *.msg and *.oft
  -f, --file <FILE>      Parse one PST, OST, MSG or OFT file, whatever its name
      --code-page <CP>   Code page of items that don't give theirs (default 1252)

Output:
  -o, --output <PATH>              Directory of CSV tables, or JSON/JSONL file
  -t, --output-type <TYPE>         csv, json, or jsonl
      --properties                 Add the Properties table: every property as stored
      --extract-attachments <DIR>  Write each attachment's data once, named by its SHA-256
  -C, --clobber                    Overwrite files when saving or extracting
```

```bash
juntei parse-file outlook -d Users -o outlook-csv
juntei parse-file outlook -f mailbox.ost -t jsonl \
  | jq 'select(.Table == "Items" and .Hidden == false) | {DeliveryTime, SenderAddress, Subject}'
juntei parse-file outlook -f suspicious.msg -o msg-csv --properties --extract-attachments isolated/
```

`-d` ค้นหาไฟล์ที่ลงท้ายด้วย `.pst`, `.ost`, `.msg` และ `.oft` โดยไม่คำนึงถึงตัวพิมพ์เล็กพิมพ์ใหญ่ ไม่ว่าจะระบุ
ไฟล์ด้วยวิธีใด ไบต์แรก ๆ ของไฟล์จะบอกว่าเป็นไฟล์ชนิดใด ได้แก่ `!BDN` สำหรับ PST หรือ OST และลายเซ็นของ
compound file สำหรับ MSG หรือ OFT ไฟล์อื่นจะถูกรายงานว่าเป็นไฟล์ที่วิเคราะห์ไม่ได้ สถานะการออก (exit status)
จะเป็น 1 เมื่ออ่านไฟล์ไม่ได้เลยแม้แต่ไฟล์เดียว

`--extract-attachments` เขียนข้อมูลของสิ่งที่แนบแต่ละรายการเพียงครั้งเดียว โดยตั้งชื่อเป็น `<SHA-256>` พร้อม
นามสกุลของสิ่งที่แนบเมื่อนามสกุลนั้นประกอบด้วยตัวอักษรและตัวเลข ASCII 1 ถึง 10 ตัว (`0c7d…1311.jpg`)
และไม่มีการใช้ชื่อใดจากรายการในพาธ ไดเรกทอรีต้องว่างเปล่า เว้นแต่จะระบุ `-C` เมื่อใช้ `-C` ไฟล์ที่มีชื่อเดียวกัน
แต่มีเนื้อหาอื่นจะถูกแทนที่

## ตารางเอาต์พุต { #output-tables }

CSV จะสร้างแปดไฟล์ หรือเก้าไฟล์เมื่อใช้ `--properties` โดยแต่ละไฟล์มีเพียงแถวส่วนหัวเมื่อไม่มีแถวข้อมูล แถว JSON
และ JSONL มี `Table` (`Files`, `Folders`, `Items`, `Recipients`, `Attachments`, `Contacts`, `Calendar`,
`Tasks` หรือ `Properties`) ต่อจาก `SourceFile` แถวต่าง ๆ ของรายการจะตามหลังรายการนั้น และแถวของรายการ
ที่ฝังอยู่จะตามหลังแถวเหล่านั้น ส่วนแถว Files ของไฟล์จะมาหลังแถวอื่นทั้งหมดของไฟล์นั้น

| ไฟล์ CSV | เนื้อหา |
|---|---|
| `Outlook-Files.csv` | หนึ่งแถวต่อหนึ่งไฟล์ที่อ่าน: ที่เก็บข้อมูลหรือไฟล์ และสิ่งที่อยู่ภายใน |
| `Outlook-Folders.csv` | โฟลเดอร์ของ PST และ OST |
| `Outlook-Items.csv` | รายการทั้งหมด ได้แก่ ข้อความของโฟลเดอร์ รายการที่ซ่อนอยู่ของโฟลเดอร์ รายการของไฟล์ `.msg` และ `.oft` และรายการที่ฝังอยู่ในสิ่งที่แนบของรายการเหล่านั้น |
| `Outlook-Recipients.csv` | ผู้รับของรายการ |
| `Outlook-Attachments.csv` | สิ่งที่แนบของรายการ |
| `Outlook-Contacts.csv` | ฟิลด์ของผู้ติดต่อและรายชื่อการแจกจ่าย |
| `Outlook-Calendar.csv` | ฟิลด์ของรายการปฏิทินและข้อความการประชุม |
| `Outlook-Tasks.csv` | ฟิลด์ของงานและคำของาน |
| `Outlook-Properties.csv` | เมื่อใช้ `--properties`: คุณสมบัติทุกตัวของที่เก็บข้อมูล โฟลเดอร์ รายการ ผู้รับ และสิ่งที่แนบ ตามที่จัดเก็บไว้ |

## ฟิลด์ { #fields }

ทุกตารางขึ้นต้นด้วยฟิลด์เหล่านี้

| ฟิลด์ | การตีความ |
|---|---|
| `SourceFile` | ไฟล์ที่อ่านแถวนั้นมา |
| `User` | ผู้ใช้ Windows ที่เป็นเจ้าของโปรไฟล์ซึ่งพาธนั้นอยู่ (โฟลเดอร์ถัดจาก `Users`) |

### ไฟล์ (Files) { #files }

| ฟิลด์ | การตีความ |
|---|---|
| `FileType` | `PST`, `OST`, `MSG`, `OFT` หรือ `CFB` สำหรับ compound file ที่อ่านเป็นรายการ แต่มี class ID ที่ไม่ใช่ของข้อความหรือของเทมเพลต (โปรแกรมอื่นนอกจาก Outlook จำนวนมากปล่อยค่านี้ว่างไว้) |
| `Format` | โครงสร้างของที่เก็บข้อมูล: `ANSI` (Outlook 97 ถึง 2002), `Unicode` (Outlook 2003 ขึ้นไป) หรือ `Unicode 4K` (OST ของ Outlook 2013 ขึ้นไป) หรือเวอร์ชันของ compound file สำหรับไฟล์ MSG หรือ OFT (`CFB v3`, `CFB v4`) |
| `Version`, `ClientVersion` | `wVer` และ `wVerClient` ในส่วนหัวของที่เก็บข้อมูล |
| `Encryption` | วิธีเข้ารหัสบล็อกของที่เก็บข้อมูล: `None`, `Permute` หรือ `Cyclic` |
| `DisplayName` | ชื่อของที่เก็บข้อมูล (`Personal Folders` หรือที่อยู่อีเมล) |
| `RecordKey` | ตัวระบุของที่เก็บข้อมูล (เลขฐานสิบหก) ซึ่งปรากฏอยู่ใน entry id ของรายการในที่เก็บข้อมูลนั้น |
| `Folders` | โฟลเดอร์ของที่เก็บข้อมูล |
| `Items` | ข้อความในโฟลเดอร์ของที่เก็บข้อมูล หรือ 1 สำหรับไฟล์ MSG หรือ OFT |
| `HiddenItems` | รายการที่ซ่อนอยู่ (ข้อมูลที่เชื่อมโยงกับโฟลเดอร์) ของโฟลเดอร์ต่าง ๆ |
| `EmbeddedItems` | รายการที่ฝังอยู่ในสิ่งที่แนบในทุกระดับ |
| `Attachments` | สิ่งที่แนบของรายการทั้งหมดเหล่านั้น |
| `Problems` | ปัญหาที่พบในไฟล์ |

### โฟลเดอร์ (Folders) { #folders }

| ฟิลด์ | การตีความ |
|---|---|
| `FolderID`, `ParentID` | node id ของโฟลเดอร์ และของโฟลเดอร์แม่ (ว่างเปล่าสำหรับโฟลเดอร์ราก) |
| `FolderPath` | ชื่อของโฟลเดอร์ตั้งแต่รากลงมา: `/Top of Personal Folders/Inbox`, `/Root - Mailbox/IPM_SUBTREE/Sent Items` |
| `Name` | ชื่อของโฟลเดอร์ |
| `ContainerClass` | สิ่งที่โฟลเดอร์เก็บไว้: `IPF.Note` (อีเมล), `IPF.Appointment`, `IPF.Contact`, `IPF.Task`, `IPF.Imap`, … |
| `ContentCount`, `UnreadCount` | จำนวนข้อความ และจำนวนข้อความที่ยังไม่ได้อ่าน ตามที่โฟลเดอร์ระบุ |
| `Messages`, `HiddenItems`, `Subfolders` | จำนวนแถวในตาราง contents, ตาราง associated contents และตาราง hierarchy ของโฟลเดอร์ ซึ่งเป็นสิ่งที่ Juntei อ่าน หากต่างจาก `ContentCount` ควรตรวจสอบเพิ่มเติม |
| `CreationTime`, `LastModifiedTime` | เวลาที่สร้างโฟลเดอร์และเวลาที่เปลี่ยนแปลงครั้งล่าสุด (UTC) หากที่เก็บข้อมูลเก็บค่าเหล่านี้ไว้ |
| `Problem` | สาเหตุที่เปิดโฟลเดอร์ไม่ได้ ในกรณีนี้จะไม่อ่านโฟลเดอร์ย่อยของโฟลเดอร์นั้น |

### รายการ (Items) { #items }

| ฟิลด์ | การตีความ |
|---|---|
| `FolderPath` | โฟลเดอร์ของรายการ ว่างเปล่าในไฟล์ MSG หรือ OFT |
| `ItemID` | id ของรายการ: node id ใน PST หรือ OST, `0` ในไฟล์ MSG หรือ OFT และสำหรับรายการที่ฝังอยู่คือ id ของรายการแม่ตามด้วยดัชนีของสิ่งที่แนบ (`2097348/0`) |
| `Hidden` | รายการนั้นเป็นข้อมูลที่เชื่อมโยงกับโฟลเดอร์ (รายการที่ซ่อนอยู่) หรือไม่ |
| `EmbeddedIn` | id ของรายการที่รายการนี้ถูกแนบอยู่ ว่างเปล่าสำหรับรายการอื่น |
| `ItemType` | ชนิดของรายการตามที่ message class ระบุ: `Mail`, `MeetingMessage`, `Appointment`, `Contact`, `DistributionList`, `Task`, `TaskRequest`, `StickyNote`, `Journal`, `Post`, `Report`, `Configuration` หรือ `Other` |
| `MessageClass` | message class ตามที่จัดเก็บไว้ (`IPM.Note`, `IPM.Note.SMIME`, `IPM.Schedule.Meeting.Request`, `REPORT.IPM.Note.NDR`, …) |
| `SubmitTime` | เวลาที่ไคลเอนต์ของผู้ส่งส่งข้อความออกไป (UTC ตามนาฬิกาของผู้ส่ง) |
| `DeliveryTime` | เวลาที่ข้อความถูกส่งถึงกล่องจดหมายนี้ (UTC) |
| `CreationTime`, `LastModifiedTime` | เวลาที่สำเนานี้ถูกสร้างในที่เก็บข้อมูลหรือไฟล์ และเวลาที่เปลี่ยนแปลงครั้งล่าสุด (UTC) |
| `Subject` | หัวเรื่อง พร้อมคำนำหน้า (`RE: `) |
| `SenderName`, `SenderAddressType`, `SenderAddress` | ผู้ส่ง: ชื่อ ชนิดของที่อยู่ (`SMTP`, `EX`) และที่อยู่ชนิดนั้น |
| `SenderSMTP-Address` | ที่อยู่ SMTP ของผู้ส่ง เมื่อจัดเก็บไว้ควบคู่กับที่อยู่ชนิดอื่น |
| `SentRepresentingName`, `SentRepresentingAddress` | ผู้ที่ข้อความถูกส่งในนาม เช่น ผู้จัดการที่ผู้รับมอบสิทธิ์ (delegate) ส่งแทน หรือกล่องจดหมายที่ใช้ร่วมกัน พร้อมที่อยู่ SMTP เมื่อจัดเก็บไว้ มิฉะนั้นเป็นที่อยู่ตามชนิดของตน |
| `To`, `Cc`, `Bcc` | ชื่อของผู้รับตามที่รายการแสดง (ดูที่อยู่ของผู้รับในตาราง Recipients) `Bcc` มีอยู่เฉพาะในสำเนาของผู้ส่ง |
| `InternetMessageID`, `InReplyTo` | `Message-ID` ของข้อความ และของข้อความที่ข้อความนี้ตอบกลับ |
| `ConversationTopic` | หัวเรื่องที่ไม่มีคำนำหน้า ซึ่งข้อความในการสนทนาเดียวกันใช้ร่วมกัน |
| `Importance`, `Sensitivity` | `Low`, `Normal` หรือ `High` และ `Normal`, `Personal`, `Private` หรือ `Confidential` |
| `Read` | รายการถูกทำเครื่องหมายว่าอ่านแล้วหรือไม่ |
| `Flags` | แฟล็กของข้อความ (`PidTagMessageFlags`, เลขฐานสิบหก): `0x1` อ่านแล้ว, `0x2` ไม่ได้แก้ไข, `0x4` ส่งออกแล้ว, `0x8` ยังไม่ได้ส่ง (ฉบับร่าง), `0x10` มีสิ่งที่แนบ, `0x20` จากฉัน, `0x40` ซ่อนอยู่, `0x80` ส่งซ้ำ, `0x400` เคยอ่านแล้ว, `0x2000` จากอินเทอร์เน็ต |
| `Size` | ขนาดของรายการในที่เก็บข้อมูล รวมสิ่งที่แนบ |
| `Recipients`, `Attachments` | จำนวนผู้รับและจำนวนสิ่งที่แนบของรายการ |
| `Categories` | หมวดหมู่ของรายการ คั่นด้วย `;` |
| `BodyPreview` | ส่วนต้นของเนื้อหาแบบข้อความธรรมดา (plain text) โดยยุบช่องว่างที่ต่อเนื่องกัน ยาวไม่เกิน 256 อักขระ ว่างเปล่าเมื่อรายการมีเพียงเนื้อหาแบบ HTML หรือ RTF |
| `Problems` | สิ่งที่อ่านไม่ได้ในรายการ (ไม่เกิน 10 รายการ ตามด้วยจำนวนที่เหลือ) |

### ผู้รับ (Recipients) { #recipients }

| ฟิลด์ | การตีความ |
|---|---|
| `ItemID` | id ของรายการ |
| `Index` | แถวของผู้รับในตารางผู้รับ (recipient table) ของรายการ เริ่มจาก 0 |
| `RecipientType` | `To`, `Cc`, `Bcc`, `Originator` หรือค่าตามที่จัดเก็บไว้ (เลขฐานสิบหก) |
| `Name`, `AddressType`, `Address` | ชื่อ ชนิดของที่อยู่ และที่อยู่ของผู้รับ |
| `SMTP-Address` | ที่อยู่ SMTP เมื่อจัดเก็บไว้ควบคู่กับที่อยู่ชนิดอื่น |

### สิ่งที่แนบ (Attachments) { #attachments }

| ฟิลด์ | การตีความ |
|---|---|
| `ItemID`, `Index` | id ของรายการ และแถวของสิ่งที่แนบในตารางสิ่งที่แนบ (attachment table) ของรายการ เริ่มจาก 0 |
| `Method` | `ByValue` (ไฟล์), `ByReference`, `ByReferenceResolve`, `ByReferenceOnly` (พาธไปยังไฟล์), `EmbeddedMessage` (รายการ), `OLE` (ออบเจกต์ OLE), `ByWebReference` หรือตัวเลข |
| `Filename` | ชื่อของสิ่งที่แนบ: ชื่อไฟล์แบบยาว ชื่อไฟล์แบบสั้น หรือชื่อที่แสดง |
| `Extension` | นามสกุลตามที่จัดเก็บไว้ (`.pdf`) มิฉะนั้นใช้นามสกุลจากชื่อ |
| `MimeType`, `ContentID` | ชนิด MIME และ content id ที่เนื้อหา HTML ใช้อ้างถึงสิ่งที่แนบนั้น |
| `Inline` | เนื้อหาแสดงสิ่งที่แนบนั้นในตัว (เช่น รูปภาพในเนื้อหา HTML) แทนการแสดงเป็นรายการสิ่งที่แนบหรือไม่ |
| `Hidden` | Outlook ซ่อนสิ่งที่แนบนั้นจากรายการสิ่งที่แนบหรือไม่ |
| `Size` | ขนาดตามที่ที่เก็บข้อมูลระบุ รวมคุณสมบัติของสิ่งที่แนบ |
| `DataSize`, `SHA256` | ขนาดและ SHA-256 ของข้อมูล ได้แก่ ไฟล์ หรือ compound file ของออบเจกต์ OLE ใน PST หรือ OST ว่างเปล่าสำหรับรายการที่ฝังอยู่และสิ่งที่แนบแบบอ้างอิง |
| `Pathname` | พาธของสิ่งที่แนบแบบอ้างอิง |
| `CreationTime`, `LastModifiedTime` | เวลาของสิ่งที่แนบ (UTC) ซึ่งมักเป็นเวลาของไฟล์ที่ใช้สร้างสิ่งที่แนบนั้น |
| `EmbeddedSubject` | หัวเรื่องของรายการที่ฝังอยู่ ซึ่งแถวของรายการนั้นเองมี `ItemID` เป็น `<ItemID>/<Index>` |
| `ExtractedFile` | เมื่อใช้ `--extract-attachments` คือไฟล์ในไดเรกทอรีนั้นที่ข้อมูลของสิ่งที่แนบถูกเขียนลงไป |
| `Problems` | สิ่งที่อ่านหรือเขียนไม่ได้ในสิ่งที่แนบ |

### ผู้ติดต่อ (Contacts) { #contacts }

| ฟิลด์ | การตีความ |
|---|---|
| `FolderPath`, `ItemID`, `ItemType` | เหมือนในตาราง Items: `Contact` หรือ `DistributionList` |
| `DisplayName`, `GivenName`, `Surname`, `Nickname` | ชื่อต่าง ๆ ของผู้ติดต่อ |
| `FileUnder` | ชื่อที่ใช้จัดเรียงผู้ติดต่อ (file under) |
| `Company`, `Title`, `Department` | ที่ทำงานของผู้ติดต่อ |
| `Email1Name`, `Email1Address`, `Email2Name`, `Email2Address`, `Email3Name`, `Email3Address` | ที่อยู่อีเมลสามรายการของผู้ติดต่อ แต่ละรายการพร้อมชื่อที่แสดง |
| `BusinessPhone`, `HomePhone`, `MobilePhone` | หมายเลขโทรศัพท์ของผู้ติดต่อ |
| `InstantMessaging` | ที่อยู่สำหรับการส่งข้อความโต้ตอบแบบทันที (instant messaging) ของผู้ติดต่อ |
| `CreationTime`, `LastModifiedTime` | เหมือนในตาราง Items |

### ปฏิทิน (Calendar) { #calendar }

| ฟิลด์ | การตีความ |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | เหมือนในตาราง Items: `Appointment` หรือ `MeetingMessage` |
| `StartTime`, `EndTime` | เวลาเริ่มต้นและสิ้นสุดของการนัดหมาย (UTC) สำหรับการนัดหมายแบบเกิดซ้ำคือเวลาของครั้งแรก |
| `DurationMinutes` | ระยะเวลาเป็นนาที |
| `Location` | สถานที่ |
| `AllDay`, `Recurring` | เป็นกิจกรรมตลอดทั้งวันหรือไม่ และเกิดซ้ำหรือไม่ (ข้อยกเว้นของการเกิดซ้ำเป็นรายการที่ฝังอยู่) |
| `BusyStatus` | `Free`, `Tentative`, `Busy`, `OutOfOffice` หรือ `WorkingElsewhere` |
| `ResponseStatus` | การตอบกลับของผู้ใช้: `None`, `Organized`, `Tentative`, `Accepted`, `Declined` หรือ `NotResponded` |
| `Organizer` | ชื่อของผู้ส่งรายการ |
| `AllAttendees`, `ToAttendees`, `CcAttendees` | ผู้เข้าร่วมตามที่รายการระบุ |
| `GlobalObjectID` | ตัวระบุของการประชุมที่ใช้ร่วมกันข้ามกล่องจดหมาย (เลขฐานสิบหก) คือการประชุมเดียวกันในปฏิทินของผู้จัดและของผู้เข้าร่วม |
| `ReminderSet`, `ReminderTime` | ตั้งการแจ้งเตือนไว้หรือไม่ และเวลาที่ตั้งการแจ้งเตือนไว้ (UTC) |
| `CreationTime`, `LastModifiedTime` | เหมือนในตาราง Items |

### งาน (Tasks) { #tasks }

| ฟิลด์ | การตีความ |
|---|---|
| `FolderPath`, `ItemID`, `ItemType`, `Subject` | เหมือนในตาราง Items: `Task` หรือ `TaskRequest` |
| `Status` | `NotStarted`, `InProgress`, `Complete`, `Waiting` หรือ `Deferred` |
| `PercentComplete` | ความคืบหน้าของงาน ตั้งแต่ 0 ถึง 100 |
| `StartTime`, `DueTime` | วันเริ่มต้นและวันครบกำหนด: วันที่ตามเขตเวลาของผู้ใช้ โดยไม่มีค่าชดเชยเวลา |
| `CompletedTime` | เวลาที่งานเสร็จสมบูรณ์ (UTC) |
| `Complete` | งานเสร็จสมบูรณ์แล้วหรือไม่ |
| `Owner`, `Assigner` | เจ้าของงาน และผู้มอบหมายงาน |
| `ReminderSet`, `ReminderTime` | เหมือนในตาราง Calendar |
| `CreationTime`, `LastModifiedTime` | เหมือนในตาราง Items |

### คุณสมบัติ (Properties) { #properties }

| ฟิลด์ | การตีความ |
|---|---|
| `ItemID` | id ของรายการ หรือ `store` สำหรับคุณสมบัติของที่เก็บข้อมูลเอง และ `folder:<FolderID>` สำหรับคุณสมบัติของโฟลเดอร์ |
| `Scope` | เจ้าของคุณสมบัติ: `Store`, `Folder`, `Item`, `Recipient` หรือ `Attachment` |
| `Index` | ดัชนีของผู้รับหรือสิ่งที่แนบ ว่างเปล่าสำหรับส่วนอื่น |
| `PropertyTag` | id และชนิดของคุณสมบัติ ในรูปแบบ `0x0037001F` (`PidTagSubject` ซึ่งเป็นสตริง) |
| `PropertyName` | สำหรับคุณสมบัติที่มีชื่อ (named property ซึ่งมี id ตั้งแต่ `0x8000`) คือชุดคุณสมบัติ (property set) และชื่อ: `PSETID_Address:0x8083`, `PS_PUBLIC_STRINGS:Keywords` หรือ GUID ของชุดคุณสมบัติเมื่อไม่ใช่ชุดที่รู้จักกันทั่วไป |
| `Type` | ชนิดของคุณสมบัติ: `PtypString`, `PtypInteger32`, `PtypTime`, `PtypBinary`, `PtypMultipleString`, … |
| `Value` | ค่า: ข้อความสำหรับสตริง, ISO 8601 UTC สำหรับเวลา (ซึ่ง `-Z` ไม่แปลง), เลขฐานสิบหกสำหรับค่าไบนารี (4,096 ไบต์แรก ตามด้วย `…(N bytes)`), ค่าของคุณสมบัติแบบหลายค่าในรูปรายการ (ใน CSV คั่นด้วย `, `), `(object)` สำหรับรายการที่ฝังอยู่หรือข้อมูลของสิ่งที่แนบ ไบต์ในรูปเลขฐานสิบหกพร้อมหมายเหตุสำหรับค่าที่เป็นชนิดที่ไม่ได้ถอดรหัส และสำหรับค่าที่เสียหาย คือส่วนที่กู้คืนได้ ตามด้วยหมายเหตุที่มีไบต์ของค่านั้นในรูปเลขฐานสิบหก (มีเพียงไบต์และหมายเหตุหากกู้คืนไม่ได้เลย) |

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียนตารางลงในหมวดหมู่ใหม่ `Communication` (การสื่อสาร) เป็น `Outlook-Files.csv`,
`Outlook-Folders.csv`, `Outlook-Items.csv`, `Outlook-Recipients.csv`, `Outlook-Attachments.csv`,
`Outlook-Contacts.csv`, `Outlook-Calendar.csv` และ `Outlook-Tasks.csv` (เป็นไฟล์ `Outlook.json` หรือ
`Outlook.jsonl` ไฟล์เดียวเมื่อใช้ `-t json` หรือ `-t jsonl`) พร้อม `Communication/Outlook-logs.txt` โดยส่ง
`--code-page` ให้กับพาร์เซอร์ แต่ไม่เพิ่มตาราง Properties และไม่แยกสิ่งที่แนบออกมา หากต้องการสิ่งเหล่านี้ให้รัน
`parse-file outlook` ไทม์ไลน์แบบรวม `-j` จะรวมทุกแถว พร้อม `Comp` และตัวระบุ `Artifact` ที่เป็น `outlook`
ส่วน `-Z` จะแปลงทุกฟิลด์ที่ลงท้ายด้วย `Time` ยกเว้น `StartTime` และ `DueTime` ของงาน

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft — \[MS-PST\]: Outlook Personal Folders (.pst) File Format](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-pst/141923d5-15ab-4ef1-a524-6dce75aae546)
- [Microsoft — \[MS-OXMSG\]: Outlook Item (.msg) File Format](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxmsg/b046868c-9fbf-41ae-9ffb-8de2bd4eec82)
- [Microsoft — \[MS-OXCMSG\]: Message and Attachment Object Protocol](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxcmsg/7fd7ec40-deec-4c06-9493-1bc06b349682)
- [Microsoft — \[MS-OXPROPS\]: Exchange Server Protocols Master Property List](https://learn.microsoft.com/en-us/openspecs/exchange_server_protocols/ms-oxprops/f6ab1613-aefe-447d-a49c-18217230b148)
- [libpff — Personal Folder File (PFF) format](https://github.com/libyal/libpff/blob/main/documentation/Personal%20Folder%20File%20(PFF)%20format.asciidoc)
- [ys-outlook-parser](https://github.com/Yamato-Security/ys-outlook-parser)
