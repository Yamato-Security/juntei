# ถังรีไซเคิล { #recycle-bin }

## สิ่งที่ถังรีไซเคิลบันทึกไว้ { #what-the-recycle-bin-records }

เมื่อผู้ใช้ลบไฟล์หรือโฟลเดอร์ใน File Explorer นั้น Windows จะย้ายรายการนั้นไปยังถังรีไซเคิลของวอลุ่มที่รายการอยู่
และเขียนเรคคอร์ดดัชนีขนาดเล็กไว้ข้างกัน ซึ่งเก็บพาธเดิมของรายการ ขนาด และเวลาที่ถูกลบ ผู้ใช้แต่ละคนมีโฟลเดอร์ถัง
ของตนเองบนแต่ละวอลุ่ม โดยตั้งชื่อตาม SID ของผู้ใช้ โฟลเดอร์นี้จึงบอกได้ด้วยว่าบัญชีใดเป็นผู้ลบรายการนั้น

| เวอร์ชัน Windows | ดัชนี | เนื้อหาที่ถูกลบ |
|---|---|---|
| Vista ขึ้นไป | ไฟล์ `$I…` หนึ่งไฟล์ต่อหนึ่งรายการ (รูปแบบเวอร์ชัน 1 บน Vista ถึง 8.1 และเวอร์ชัน 2 บน Windows 10 ขึ้นไป) | `$R…` ซึ่งมีชื่อส่วนที่อยู่หลัง `$R` เหมือนกับไฟล์ดัชนี: ตัวไฟล์ หรือโฟลเดอร์พร้อมเนื้อหาภายใน |
| NT 4.0, 2000, XP, Server 2003 | ไฟล์ `INFO2` หนึ่งไฟล์ต่อหนึ่งถัง: เรคคอร์ดขนาด 800 ไบต์ที่มีทั้งพาธ ANSI และพาธ Unicode | `D<drive letter><index><extension>` เช่น `Dc1.txt` |
| 95, 98, ME | ไฟล์ `INFO2` หนึ่งไฟล์ (`INFO` บน Windows 95): เรคคอร์ดขนาด 280 ไบต์ที่มีเพียงพาธ ANSI | ชื่อแบบ `D…` เช่นเดียวกัน |

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> บัญชีที่เป็นเจ้าของถังซึ่งมีเรคคอร์ดนี้ได้ย้ายรายการที่อยู่
ณ พาธเดิมไปยังถังรีไซเคิล ณ เวลาที่บันทึกไว้ การลบด้วย Shift+Delete การลบจากบรรทัดคำสั่ง หรือการลบจาก
โปรแกรมส่วนใหญ่จะข้ามถังรีไซเคิลไปและไม่ทิ้งเรคคอร์ดใดไว้

## ตำแหน่งและการเก็บรวบรวม { #locations-and-collection }

```text
C:\$Recycle.Bin\<SID>\$I…        Windows Vista and later (each volume has its own bin)
C:\$Recycle.Bin\<SID>\$R…        the deleted file or folder
C:\RECYCLER\<SID>\INFO2          Windows NT 4.0, 2000, XP and Server 2003 (NTFS)
C:\RECYCLED\INFO2                Windows 95, 98 and ME, and FAT volumes (INFO on Windows 95)
```

ให้เก็บรวบรวมไฟล์ `$I` หรือ INFO2 จากถังของทุกวอลุ่ม หากเก็บไฟล์ `$R` และ `D…` มาด้วย จะเห็นได้ว่ารายการที่ถูกลบ
รายการใดยังอยู่ในถัง หากไม่มีไฟล์เหล่านี้ `RecycledFileFound` จะเป็น `false` สำหรับทุกรายการ เนื้อหาที่ถูกลบ
เป็นหลักฐานในตัวเองและอาจเป็นมัลแวร์ได้ ให้จัดการด้วยความระมัดระวังตามนั้น

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- `DeletedTime` คือเวลาที่รายการถูกย้ายไปยังถัง ตามนาฬิการะบบ ในรูปแบบ FILETIME แบบ UTC ไม่ใช่เวลาที่
  ถังถูกล้าง เวลาสร้างและเวลาแก้ไขของตัวรายการเองอยู่ใน `$MFT` และไฟล์ `$R`
- SID คือบัญชีที่ลบรายการนั้น ไม่จำเป็นต้องเป็นเจ้าของไฟล์ SID ที่ไม่มีโปรไฟล์อยู่ใน `ProfileList` ของไฮฟ์
  SOFTWARE (บัญชีที่ถูกลบไปแล้ว หรือบัญชีโดเมนที่ไม่เคยเข้าสู่ระบบ) จะไม่มี `User`
- เมื่อถังถูกล้าง หรือเมื่อรายการถูกคืนค่า (restore) หรือถูกลบออกจากถัง Windows Vista ขึ้นไปจะลบไฟล์ `$I`
  และ `$R` ของรายการนั้น ชื่อของไฟล์เหล่านี้อาจยังหลงเหลืออยู่ใน USN Journal, `$MFT` และ slack ของ `$I30`
  แต่ Juntei ไม่ได้กู้คืนเรคคอร์ด `$I` ที่ถูกลบจากแหล่งเหล่านี้ ส่วน INFO2 จะเก็บเรคคอร์ดดังกล่าวไว้และล้าง
  ไบต์แรกของพาธ ANSI ในกรณีนี้ `RemovedFromBin` จะเป็น `true` และ Juntei จะเติมอักขระนั้นกลับคืนจากหมายเลขไดรฟ์
- `FileSize` ในไฟล์ `$I` คือขนาดของรายการ สำหรับโฟลเดอร์คือขนาดรวมของเนื้อหาภายใน ส่วนใน INFO2 คือพื้นที่
  ที่รายการใช้บนดิสก์ โดยนับเป็นคลัสเตอร์เต็ม (4,096 ไบต์สำหรับไฟล์ขนาดเล็ก)
- พาธ ANSI ของ INFO2 มักอยู่ในรูปแบบ 8.3 และต้องใช้ code page ของระบบ ให้ใช้ `--code-page 932` สำหรับ
  Windows ภาษาญี่ปุ่น และ 936 หรือ 950 สำหรับภาษาจีน ส่วนพาธ Unicode (ถ้ามี) คือ `OriginalPath`
- ระบบ Windows Vista บางระบบเขียนไฟล์ `$I` ขนาด 543 ไบต์ ซึ่งฟิลด์ขนาดสั้นไปหนึ่งไบต์ Juntei จะอ่านเวลา
  และพาธของไฟล์เหล่านี้ และปล่อย `FileSize` ว่างไว้ พร้อมรายงานเป็นปัญหา
- `RecycledFileFound` อธิบายชุดข้อมูลที่เก็บรวบรวมมา ไม่ใช่เครื่องปลายทาง: `false` อาจหมายความว่าเครื่องมือ
  เก็บรวบรวมไม่ได้เก็บไฟล์ `$R` มา หรือรายการนั้นไม่อยู่ในถังแล้ว
- ถังบนวอลุ่มที่ไม่มีการติดตั้ง Windows จะใช้ชื่อผู้ใช้จากการติดตั้ง Windows ในชุดข้อมูลที่เก็บรวบรวมมา
  หากมีการติดตั้งอยู่เพียงชุดเดียวพอดี

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file recycle-bin <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $I files and INFO2
  -f, --file <FILE>      Parse one $I or INFO2 file (told apart by its contents if renamed)
      --code-page <CP>   INFO2's ANSI code page (default 1252; Japanese commonly 932)

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-file recycle-bin -d evidence/ -o recycle-bin.csv
juntei parse-file recycle-bin -d xp-evidence/ --code-page 932 -o recycle-bin-ja.csv
juntei parse-file recycle-bin -f '$IABC123.docx' -t jsonl | jq .
```

เมื่อใช้ `-d` นั้น Juntei จะอ่านไฟล์ที่มีชื่อเป็น `$I` ตามด้วยตัวอักษรหรือตัวเลขหกตัว (นามสกุลใดก็ได้) ไฟล์ `INFO2`
และไฟล์ `INFO` ในโฟลเดอร์ `RECYCLED` ส่วนเมื่อใช้ `-f` จะอ่านไฟล์นั้นไม่ว่าจะมีชื่ออะไร

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | ความหมาย |
|---|---|
| `SourceFile` | ไฟล์ `$I` หรือ INFO2 |
| `DeletedTime` | เวลาที่รายการถูกย้ายไปยังถัง (UTC) |
| `OriginalPath` | ตำแหน่งเดิมของรายการ สำหรับ INFO2 จะเป็นพาธ Unicode หากเรคคอร์ดมี มิฉะนั้นจะเป็นพาธ ANSI |
| `FileSize` | `$I`: ขนาดของรายการเป็นไบต์ (ขนาดรวมสำหรับโฟลเดอร์) INFO2: พื้นที่บนดิสก์ นับเป็นคลัสเตอร์เต็ม จะว่างเมื่ออ่านค่าไม่ได้ |
| `User`, `SID` | SID ของโฟลเดอร์ถัง และผู้ใช้ตามชื่อโปรไฟล์ของ SID นั้นใน `ProfileList` |
| `RecycledFile`, `RecycledFileFound` | พาธของเนื้อหาที่ถูกลบซึ่งอยู่ข้างไฟล์ดัชนี (`$R…`, `D<drive><index><extension>`) และเนื้อหานั้นอยู่ในชุดข้อมูลที่เก็บรวบรวมมาหรือไม่ |
| `Format`, `FormatVersion` | `$I` 1 (Vista ถึง 8.1) หรือ 2 (Windows 10 ขึ้นไป) และ `INFO2` 0 (95), 2 (NT 4.0), 4 (98) หรือ 5 (ME, 2000 ถึง Server 2003) |
| `RecordIndex`, `Drive`, `RecordOffset` | INFO2: หมายเลขของเรคคอร์ด (หมายเลขในชื่อไฟล์ `D…`) ไดรฟ์ และตำแหน่งที่เรคคอร์ดเริ่มต้นใน INFO2 พาธ UNC ไม่มีไดรฟ์ |
| `RemovedFromBin` | INFO2: เป็น `true` เมื่อรายการถูกคืนค่าหรือถูกลบออกจากถังแล้ว |
| `ShortPath` | INFO2: พาธ ANSI เมื่อแตกต่างจาก `OriginalPath` (มักเป็นชื่อแบบ 8.3) |
| `Problems` | ความเสียหาย ได้แก่ ไฟล์ที่ถูกตัดทอนหรือยาวเกิน หมายเลขไดรฟ์ที่ไม่ถูกต้อง พาธที่ไม่ใช่ UTF-16 ที่ถูกต้อง และเวลาการลบที่ไม่ใช่ค่าเวลา |

JSON และ JSONL มีฟิลด์เดียวกัน โดยตัวเลขและค่าบูลีนมีชนิดข้อมูลตามจริง ค่าที่เรคคอร์ดไม่มีจะเป็น `null` และ
`Problems` เป็นรายการ (list)

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียน `FileExistence/RecycleBin.csv` (`RecycleBin.json` หรือ `RecycleBin.jsonl` เมื่อใช้
`-t json` หรือ `-t jsonl`) พร้อม `FileExistence/RecycleBin-logs.txt` ตัวเลือก `--code-page` ของคำสั่งนี้ใช้กับ
INFO2 เมื่อวิเคราะห์จากดิสก์อิมเมจ ไฟล์ `$I` และ INFO2 จะถูกคัดลอก ส่วนไฟล์ `$R` และ `D…` จะเป็นไฟล์แทนที่ว่าง
`RecycledFileFound` จึงยังคงมีค่า ไทม์ไลน์แบบรวม `-j` จะรวมทุกแถว พร้อมตัวระบุ `Artifact` ที่เป็น `recycle-bin`
ส่วน `-Z` จะแปลง `DeletedTime` และ [`super-timeline`](../commands/super-timeline.md) จะสร้างเหตุการณ์
`Recycle Bin deletion (moved to the bin)` หนึ่งเหตุการณ์ต่อหนึ่งรายการ

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงลำดับตาม `DeletedTime` แยกตามผู้ใช้ แล้วเปรียบเทียบกับเวลาของกิจกรรมที่กำลังตรวจสอบ
2. ค้นหา `OriginalPath` ใน LNK, Jump Lists, ShellBags และ RecentDocs เพื่อดูว่ารายการนั้นถูกใช้งานก่อนถูกลบหรือไม่
3. ใน USN Journal การลบจะปรากฏเป็นการเปลี่ยนชื่อ (rename) เข้าไปใน `$Recycle.Bin` ในช่วงเวลาใกล้เคียงกับ
   `DeletedTime` ส่วน `$MFT` ให้เวลาของตัวรายการเอง
4. เมื่อ `RecycledFileFound` เป็น `true` ให้คำนวณแฮชและตรวจสอบเนื้อหา `$R` หรือ `D…`
5. หากถังถูกล้างไปแล้ว ให้ค้นหาชื่อ `$I` และ `$R` ใน USN Journal และ `$MFT`

## รายละเอียดเพิ่มเติม { #further-details }

- [libyal dtformats — Windows Recycle.Bin file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycle.Bin%20file%20formats.asciidoc)
- [libyal dtformats — Windows Recycler file formats](https://github.com/libyal/dtformats/blob/main/documentation/Windows%20Recycler%20file%20formats.asciidoc)
- [Plaso — Recycle Bin and INFO2 parsers](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/recycler.py)
- [Velociraptor — Windows.Forensics.RecycleBin](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RecycleBin.yaml)
- [rifiuti2](https://github.com/abelcheung/rifiuti2)
