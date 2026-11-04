# บูตเซกเตอร์ (`$Boot`) { #boot-sector-boot }

## `$Boot` คืออะไร { #what-boot-is }

`$Boot` คือ MFT entry 7 ของทุกวอลุ่ม NTFS ซึ่งเป็น 8 KiB แรกของวอลุ่ม เซกเตอร์แรกของไฟล์นี้คือ
**volume boot record** ซึ่งเก็บ BIOS parameter block ที่อธิบายระบบไฟล์ไว้:

- ขนาดเซกเตอร์และคลัสเตอร์ จำนวนเซกเตอร์ในวอลุ่ม และ **hidden sectors**
  (เซกเตอร์บนดิสก์ที่อยู่ก่อนวอลุ่ม ซึ่งก็คือตำแหน่งที่พาร์ทิชันเริ่มต้น)
- ตำแหน่งเริ่มต้นของ `$MFT` และ `$MFTMirr` และขนาดของเรคคอร์ด FILE และ INDX
- **หมายเลขซีเรียลของวอลุ่ม (volume serial number)** ขนาด 64 บิต ซึ่งกำหนดขึ้นเมื่อฟอร์แมตวอลุ่ม

ส่วนที่เหลือของเซกเตอร์แรกและ 15 เซกเตอร์ถัดไปเก็บ **boot code** ที่ใช้โหลด
Windows

32 บิตล่างของหมายเลขซีเรียลคือค่าที่ Windows แสดงเป็น `XXXX-XXXX` (`vol`, `dir`) และเป็นค่าที่
[ทางลัด LNK](lnk.md), [Jump Lists](jump-lists.md) และ [Prefetch](prefetch.md) บันทึกไว้สำหรับ
วอลุ่มที่ไฟล์อยู่ `$Boot` จึงเชื่อมโยงเรคคอร์ดเหล่านั้นกับวอลุ่มของโฮสต์ที่เก็บรวบรวมมาได้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> หมายเลขซีเรียลของวอลุ่มใน LNK, Jump List หรือ Prefetch
ที่ตรงกับ `VolumeSerialNumber` ของ `$Boot` ชี้ไปยังวอลุ่มนั้น หรือวอลุ่มอื่นที่ถูกกำหนด หรือถูกคัดลอกมา
พร้อมหมายเลขซีเรียลเดียวกัน หมายเลขซีเรียลสามารถเปลี่ยนได้และไม่ได้ไม่ซ้ำกันระหว่างโฮสต์

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\$Boot          (MFT entry 7; the first 8 KiB of the volume)
```

ให้เก็บรวบรวมด้วยเครื่องมือนิติวิทยาศาสตร์ที่อ่านข้อมูลดิบได้ หรืออ่านเซกเตอร์แรกของอิมเมจวอลุ่ม ซึ่งทั้งสองแบบคือ
ไบต์ชุดเดียวกัน Juntei อ่านไฟล์ใดก็ได้ที่เริ่มต้นด้วยบูตเซกเตอร์ NTFS จึงใช้อิมเมจวอลุ่ม
เป็นอินพุตได้ ส่วนอิมเมจดิสก์จะเริ่มต้นด้วยตารางพาร์ทิชันแทน ให้ส่งวอลุ่มของพาร์ทิชันให้ Juntei

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- หมายเลขซีเรียลเปลี่ยนเมื่อฟอร์แมตวอลุ่ม ไฟล์ LNK ที่หมายเลขซีเรียลไม่ตรงกับ
  วอลุ่มปัจจุบันใดเลยอาจชี้ไปยังสื่อแบบถอดได้ เซิร์ฟเวอร์ของแชร์บนเครือข่าย หรือวอลุ่มที่ถูกฟอร์แมต
  หลังจากนั้น
- boot code จะเหมือนกันบนวอลุ่มที่ฟอร์แมตโดย Windows เดียวกัน โดยเหมือนกันทุกประการในไฟล์ `$Boot` ของ
  Windows 10, 11 และ Server 2012 R2 ใน DFIR Artifact Museum bootkit ประเภท volume boot record
  จะเปลี่ยน boot code หรือฟิลด์ hidden sectors ซึ่ง boot code ใช้ค้นหาส่วนที่เหลือของ
  `$Boot` ให้เปรียบเทียบ `BootCodeSHA256`, `JumpInstruction` และ `HiddenSectors` กับโฮสต์อื่นและกับ
  ตารางพาร์ทิชันก่อนสรุปสิ่งใด เพราะ boot manager หรือซอฟต์แวร์เข้ารหัสก็สามารถเปลี่ยน boot
  code ได้เช่นกัน
- อิมเมจของวอลุ่มที่เข้ารหัสด้วย BitLocker จะเริ่มต้นด้วยบูตเซกเตอร์ของ BitLocker เอง (OEM ID
  `-FVE-FS-`) เมื่อส่งด้วย `-f` Juntei จะรายงานบูตเซกเตอร์นั้น และ `Problems` จะระบุว่าไม่ใช่ NTFS บูต
  เซกเตอร์ NTFS อยู่ในข้อมูลที่เข้ารหัส `$Boot` ที่เก็บรวบรวมจากวอลุ่มที่ปลดล็อกแล้วจึงเป็นของ NTFS
- `Problems` แสดงความไม่สอดคล้องเชิงโครงสร้างและการถูกตัดทอนที่ตรวจพบได้ แต่ไม่ได้เป็นการทดสอบความสามารถในการเมานต์ของ Windows อย่างครบถ้วน ให้ใช้เพื่อประเมินเซกเตอร์ที่ได้จากการ carving และสำเนา ไม่ใช่ใช้เป็นข้อค้นพบการโจมตีในตัวเอง แฮชของ boot code ไม่รวม jump เริ่มต้น BIOS parameter block และ end signature ให้ตรวจสอบฟิลด์เหล่านั้นด้วย

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs boot <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $Boot files (named $Boot, or starting with an NTFS boot sector)
  -f, --file <FILE>      Parse one $Boot file, or an NTFS volume image (its first sector)

Output:
  -o, --output <FILE>       Save every boot sector
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs boot -f '$Boot'
juntei parse-ntfs boot -d evidence/ -o boot.csv
juntei parse-ntfs boot -f volume.dd -t json
```

สรุปผลจะแสดงหมายเลขซีเรียลของวอลุ่มของแต่ละบูตเซกเตอร์ (ทั้งในรูปแบบที่ Windows แสดง และรูปแบบที่ฟิลด์ของ LNK และ
Prefetch แสดง) ขนาด ขนาดคลัสเตอร์ และคลัสเตอร์ของ `$MFT` เมื่อวิเคราะห์หลายโฮสต์ จะ
จัดกลุ่มตามแฮชของ boot code เพื่อให้วอลุ่มที่มี boot code ต่างจากวอลุ่มอื่นโดดเด่นขึ้นมา

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `SourcePath` | ไฟล์อินพุต |
| `VolumeSerialNumber` | หมายเลขซีเรียล 32 บิต ตามที่เอาต์พุตของ LNK, Jump List และ Prefetch แสดง (`180B1E67`) Windows แสดงเป็น `180B-1E67` |
| `VolumeSerialNumber64` | หมายเลขซีเรียล 64 บิตทั้งหมด |
| `OEM-ID` | `NTFS` บนวอลุ่ม NTFS |
| `BytesPerSector`, `SectorsPerCluster`, `ClusterSize` | เรขาคณิตของวอลุ่ม จำนวนเซกเตอร์ต่อคลัสเตอร์ที่มากกว่า 128 จะถูกบันทึกเป็นเลขยกกำลังของสอง ซึ่งถอดรหัสไว้ที่นี่ ว่างเมื่อค่าไม่ถูกต้อง |
| `TotalSectors`, `VolumeSize` | จำนวนเซกเตอร์ในวอลุ่มและขนาดเป็นไบต์ ไม่รวมสำเนาของบูตเซกเตอร์ที่ท้ายวอลุ่ม |
| `HiddenSectors` | เซกเตอร์บนดิสก์ที่อยู่ก่อนวอลุ่ม คือตำแหน่งที่พาร์ทิชันเริ่มต้น |
| `MFT-Cluster`, `MFT-Offset` | ตำแหน่งเริ่มต้นของ `$MFT` เป็นคลัสเตอร์และเป็นไบต์นับจากจุดเริ่มต้นของวอลุ่ม |
| `MFT-MirrorCluster`, `MFT-MirrorOffset` | ตำแหน่งเริ่มต้นของ `$MFTMirr` |
| `FileRecordSize`, `IndexRecordSize` | จำนวนไบต์ต่อเรคคอร์ด FILE (โดยทั่วไป 1,024) และต่อเรคคอร์ด INDX (โดยทั่วไป 4,096) |
| `MediaDescriptor` | `F8` สำหรับฮาร์ดดิสก์ |
| `JumpInstruction` | สามไบต์แรก (`EB 52 90`) คือการ jump ข้าม BIOS parameter block ไปยัง boot code |
| `EndSignature` | `55 AA` ที่ท้ายบูตเซกเตอร์ที่ถูกต้อง |
| `BootCodeSHA256` | SHA-256 ของ boot code ได้แก่ ไบต์ 0x54 ถึง 0x1FD ของเซกเตอร์แรกและส่วนที่เหลือของ 8 KiB |
| `Problems` | สิ่งที่ทำให้ Windows ไม่เมานต์ ได้แก่ OEM ID ที่ไม่ใช่ `NTFS`, ไม่มี `55 AA`, ขนาดไม่ถูกต้อง, ฟิลด์ FAT ที่ NTFS ต้องเป็นศูนย์ถูกตั้งค่า, `$MFT` เลยท้ายวอลุ่ม และไฟล์ที่สั้นกว่า 8 KiB |

JSON/JSONL มีฟิลด์และชื่อเหมือนกัน โดยตัวเลขเป็นชนิดตัวเลข ค่าที่ไม่ทราบเป็น `null` และ
`Problems` เป็นลิสต์

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. นำ `VolumeSerialNumber` ของแต่ละโฮสต์มาวางคู่กับหมายเลขซีเรียลของวอลุ่มในไฟล์ LNK, Jump
   Lists และไฟล์ Prefetch รายการที่ไม่ตรงกับวอลุ่มที่เก็บรวบรวมมาใดเลยคือวอลุ่มอื่น (ไดรฟ์ USB
   แชร์ วอลุ่มที่ถูกฟอร์แมตหลังจากนั้น)
2. ค้นหาหมายเลขซีเรียลอื่น ๆ ในเอาต์พุตของ [อุปกรณ์ USB](usb.md) ซึ่งแสดงหมายเลขซีเรียลของวอลุ่ม
   แบบถอดได้ที่เคยเมานต์
3. เปรียบเทียบ `BootCodeSHA256` และ `HiddenSectors` ระหว่างโฮสต์ ตรวจสอบ boot code ที่แตกต่าง
   ด้วย disassembler และตารางพาร์ทิชันของดิสก์ และมองหารายการ event log ที่เกี่ยวกับการบูต
4. ใช้ `MFT-Offset` และ `ClusterSize` เพื่อค้นหา `$MFT` ในอิมเมจวอลุ่ม

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: ภาพรวม NTFS](https://learn.microsoft.com/en-us/windows-server/storage/file-server/ntfs-overview)
- [libfsntfs: เอกสารรูปแบบ NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (volume header)
- [The Sleuth Kit: fsstat](https://www.sleuthkit.org/sleuthkit/man/fsstat.html)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `boot`)
