# RecentDocs { #recentdocs }

## RecentDocs คืออะไร { #what-recentdocs-is }

Windows Explorer เก็บรายการไฟล์และโฟลเดอร์ที่ใช้ล่าสุด (most-recently-used) ไว้ใน `NTUSER.DAT` ของผู้ใช้แต่ละราย
รายการระดับรากครอบคลุมรายการล่าสุดทุกชนิด ส่วนซับคีย์จัดกลุ่มตามนามสกุล เช่น `.docx` และมีรายการ
`Folder` ด้วย `MRUListEx` ระบุลำดับของค่าที่มีหมายเลขกำกับ

RecentDocs มีประโยชน์ในการระบุเอกสารที่ผู้ใช้มีปฏิสัมพันธ์ด้วย รวมถึงไฟล์ที่ภายหลัง
ถูกลบหรือย้ายไปแล้ว ค่าต่าง ๆ ยังอาจมีชื่อทางลัดที่ Windows ใช้ในโฟลเดอร์ Recent ของผู้ใช้
ซึ่งสามารถเชื่อมโยงกับหลักฐาน LNK ได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Explorer ได้บันทึกรายการไว้ในรายการ
RecentDocs ของผู้ใช้รายนั้น เวลาของคีย์ระบุวันเวลาได้เฉพาะรายการล่าสุดในรายการนั้นเท่านั้น

## ตำแหน่ง { #location }

```text
C:\Users\<user>\NTUSER.DAT
Software\Microsoft\Windows\CurrentVersion\Explorer\RecentDocs
```

ให้เก็บรวบรวม `NTUSER.DAT` และ transaction log ของทุกโปรไฟล์ที่เกี่ยวข้อง รวมถึงไดเรกทอรี LNK
ใน Recent ของผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- `MRUListEx` ให้ลำดับ ไม่ใช่ timestamp ของทุกรายการ
- เวลาเขียนล่าสุดของคีย์สอดคล้องกับการอัปเดตครั้งล่าสุด มีเพียงตำแหน่ง MRU 0 เท่านั้นที่ได้รับ
  `LastOpenedUTC` ที่อนุมานจากเวลานั้นได้อย่างมีเหตุผลรองรับ
- รายการหนึ่งอาจปรากฏทั้งในรายการระดับรากและรายการตามนามสกุล อย่านับซ้ำเป็นการเปิดสองครั้ง
- พฤติกรรมของแอปพลิเคชันและ Explorer อาจแตกต่างกัน วิธีการเข้าถึงบางอย่างอาจไม่อัปเดต RecentDocs
- เครื่องมือทำความสะอาด/ความเป็นส่วนตัวและขีดจำกัดขนาดของรายการจะลบหลักฐานที่เก่ากว่าออกไป
- ชื่อไฟล์อาจถูกผู้โจมตีควบคุม และไม่ใช่ข้อพิสูจน์ชนิดของเนื้อหา
- การอนุมานผู้ใช้มาจากพาธของโปรไฟล์หรือข้อมูล shell folder ให้ตรวจทานชื่อผู้ใช้ที่ว่างเปล่า/กำกวม

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry recent-docs <INPUT> [OPTIONS]

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
juntei parse-registry recent-docs -f NTUSER.DAT -o recentdocs.csv
juntei parse-registry recent-docs -d evidence/ -t jsonl \
  | jq 'select(.Extension == ".docx" or .Extension == ".pdf")'
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host` | โฮสต์ที่ได้จากบริบทของการเก็บรวบรวม/ระบบเมื่อมีข้อมูล |
| `User` | เจ้าของโปรไฟล์ที่อนุมานจากพาธหรือพาธของ shell folder ให้คงความไม่แน่นอนไว้เมื่อค่าว่างเปล่า |
| `SourcePath` | `NTUSER.DAT` ที่เป็นอินพุต จำเป็นเมื่อรวมข้อมูลของผู้ใช้หลายราย |
| `Extension` | รายการย่อย (`.docx`, `Folder` เป็นต้น) เป็น null สำหรับรายการ RecentDocs ระดับราก |
| `MRU-Position` | ศูนย์คือรายการล่าสุดในรายการนี้ เป็นลำดับสัมพัทธ์ ไม่ใช่เวลาที่ผ่านไป |
| `ValueIndex` | ค่ารีจิสทรีที่มีหมายเลขซึ่ง `MRUListEx` อ้างอิงถึง ใช้เป็นจุดเชื่อมโยงเชิงโครงสร้างเท่านั้น |
| `Filename` | ชื่อรายการที่ถอดรหัสแล้ว ให้เชื่อมโยงพาธเต็มผ่าน LNK, Jump List, MFT หรือข้อมูลของแอปพลิเคชัน |
| `LinkName` | ชื่อทางลัดที่เข้ารหัสไว้ใน shell item ซึ่งมักพบในไดเรกทอรี Recent ของผู้ใช้ |
| `KeyLastWrittenUTC` | เวลาที่คีย์ของรายการเปลี่ยนแปลงครั้งล่าสุด ใช้กับรายการทั้งรายการ ไม่ใช่ทุกแถว |
| `LastOpenedUTC` | มีค่าเฉพาะสำหรับตำแหน่ง MRU 0 ซึ่งเวลาของคีย์สนับสนุนการตีความนั้น |
| `ValueDataRaw` | ไบต์ดิบเมื่อถอดรหัสไม่สำเร็จหรือหลักฐานต้องได้รับการตรวจสอบ |
| `TransactionLogStatus`, `LogsApplied` | log ของรีจิสทรีถูกเล่นซ้ำหรือไม่ และ log ใดบ้าง |
| `Problems` | ค่า MRU ที่ขาดหาย ค่าที่ไม่อยู่ในรายการ ชนิดที่ไม่ถูกต้อง หรือความเสียหายในการถอดรหัส |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. แยกรายการระดับรากและรายการตามนามสกุลของผู้ใช้แต่ละราย และตัดรายการซ้ำอย่างระมัดระวัง
2. ใช้ตำแหน่ง MRU สำหรับการเรียงลำดับ และใช้เฉพาะ `LastOpenedUTC` สำหรับข้อกล่าวอ้างที่มี timestamp
3. จับคู่ `LinkName`/`Filename` กับ LNK ใน Recent, Jump List, `$MFT` และข้อมูล USN
4. ตรวจทานชื่อล่อลวง นามสกุลซ้อน เป้าหมายบนเครือข่าย/สื่อแบบถอดได้ และเอกสารที่อยู่ใกล้ช่วงการเข้าถึงครั้งแรก
5. เชื่อมโยงกับ Prefetch/SRUM ของแอปพลิเคชันและ event log เมื่อการเรียกใช้งานมีความสำคัญ

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [libyal libfwsi](https://github.com/libyal/libfwsi)
- [RECmd และ Registry Explorer โดย Eric Zimmerman](https://github.com/EricZimmerman/RECmd)
- [โปสเตอร์ SANS Windows Forensic Analysis](https://www.sans.org/posters/windows-forensic-analysis/)
