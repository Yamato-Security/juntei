# แคชบิตแมปของ RDP { #rdp-bitmap-cache }

## สิ่งที่แคชบิตแมปของ RDP บันทึกไว้ { #what-the-rdp-bitmap-cache-records }

เมื่อไคลเอนต์ Remote Desktop (`mstsc.exe`) เชื่อมต่อโดยเปิดการแคชบิตแมปแบบถาวร (persistent bitmap caching)
ไว้ ซึ่งเป็นค่าเริ่มต้น ไคลเอนต์จะบันทึกชิ้นส่วนของหน้าจอระยะไกลลงดิสก์ เพื่อที่เซสชันครั้งต่อไปจะไม่ต้องรับชิ้นส่วนเหล่านั้นซ้ำอีก แต่ละชิ้นเป็นไทล์ (tile) ขนาดไม่เกิน 64 × 64 พิกเซล เมื่อรวมกันแล้ว ไทล์เหล่านี้คือเศษเสี้ยวเล็ก ๆ
ของสิ่งที่ผู้ใช้เห็นบนคอมพิวเตอร์ระยะไกล ได้แก่ หน้าต่าง ข้อความ ชื่อไฟล์ คำสั่ง และเดสก์ท็อป

| ไคลเอนต์ | ไฟล์แคช | ไทล์ |
|---|---|---|
| Windows 7 ขึ้นไป | `Cache0000.bin`, `Cache0001.bin`, … ไฟล์ละไม่เกิน 100 MB | 32 บิต เรียงต่อกันไปทีละไทล์ |
| Windows XP, Server 2003 และรุ่นก่อนหน้า | `bcache2.bmc` (8 บิตต่อพิกเซล), `bcache22.bmc` (16), `bcache23.bmc` (24) และ `bcache24.bmc` (32) ไฟล์ละไม่เกิน 10 MB | หนึ่งไทล์ต่อหนึ่งช่องขนาด 64 × 64 บางไทล์ถูกบีบอัด |

Windows 10 ขึ้นไปยังทิ้งไฟล์ `bcache24.bmc` ที่ว่างเปล่าไว้ข้างไฟล์ `.bin` ด้วย

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ผู้ใช้ที่เป็นเจ้าของโปรไฟล์ซึ่งมีแคชนี้ได้ใช้ไคลเอนต์
Remote Desktop เชื่อมต่อไปยังคอมพิวเตอร์ที่แสดงเนื้อหาที่ถูกแคชไว้ แคชไม่ได้บอกว่าเป็นคอมพิวเตอร์เครื่องใด
หรือเชื่อมต่อเมื่อใด

## ตำแหน่งและการเก็บรวบรวม { #locations-and-collection }

```text
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\Cache????.bin    Windows 7 and later
%LOCALAPPDATA%\Microsoft\Terminal Server Client\Cache\bcache24.bmc     (empty on Windows 10 and later)
%USERPROFILE%\Local Settings\Application Data\Microsoft\Terminal Server Client\Cache\bcache*.bmc    Windows XP and earlier
```

ค่า `BitmapPersistCacheLocation` ของ `HKCU\Software\Microsoft\Terminal Server Client` สามารถย้ายตำแหน่งของแคชได้ ให้เก็บรวบรวมโฟลเดอร์ `Cache` ทั้งโฟลเดอร์จากทุกโปรไฟล์ รวมถึงจาก Volume Shadow Copy ด้วย เพราะสำเนาที่เก่ากว่าของไฟล์แคชอาจมีไทล์ที่ไฟล์ปัจจุบันไม่มีแล้ว

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- ไทล์ไม่มีเวลา และแคชไม่ได้ระบุชื่อคอมพิวเตอร์ที่ไทล์มาจาก ให้เชื่อมโยงไทล์กับการเชื่อมต่อ RDP โดยใช้หลักฐานอื่น ได้แก่ คีย์รีจิสทรี `Terminal Server Client` ของไคลเอนต์ (`parse-registry remote-access`),
  `Default.rdp` และไฟล์ `.rdp` อื่น ๆ, Jump Lists ของ `mstsc.exe`, เวลาของไฟล์แคชใน `$MFT` และเหตุการณ์การเข้าสู่ระบบบนคอมพิวเตอร์ระยะไกล
- ไฟล์แคชหนึ่งไฟล์มีไทล์จากหลายเซสชันและจากคอมพิวเตอร์ระยะไกลหลายเครื่องปะปนกัน ไคลเอนต์จะแทนที่ไทล์เก่า
  เมื่อไฟล์เต็ม ไทล์หนึ่งจึงอาจมีอายุหลายวันหรือหลายเดือน
- ไทล์เป็นชิ้นส่วนของหน้าจอ ไม่ใช่ทั้งหน้าจอ ไทล์ที่อยู่ติดกันบนหน้าจอมักอยู่ติดกันในไฟล์ด้วย แต่ไม่เสมอไป
  ให้ประกอบหน้าจอขึ้นใหม่ด้วย [RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher) หรือด้วยตาจากภาพคอลลาจ
- ไทล์ส่วนใหญ่มีขนาด 64 × 64 พิกเซล ไทล์ที่ขอบขวาและขอบล่างของหน้าจอจะแคบกว่าหรือเตี้ยกว่า ขึ้นอยู่กับความละเอียดของหน้าจอ เช่น กว้าง 58 หรือ 62 พิกเซล หรือสูง 28 หรือ 35 พิกเซล Juntei อ่านแต่ละไทล์ตามขนาดจริงของไทล์นั้น เครื่องมือที่อ่านทุกไทล์ว่ากว้าง 64 พิกเซลจะแสดงไทล์ที่แคบออกมาเบี้ยว
- ไทล์สีเดียว (`SingleColor`) ส่วนใหญ่เป็นพื้นหลัง ให้กรองไทล์เหล่านี้ออกเพื่อค้นหาเนื้อหา
- ในไฟล์ `.bmc` ไทล์ 8 บิตใช้สีจากพาเลตต์ของเซสชัน ซึ่งแคชไม่ได้เก็บไว้ Juntei ใช้พาเลตต์เริ่มต้นของ Windows
  สีของไทล์เหล่านี้จึงอาจไม่ถูกต้อง ไทล์ที่ถูกบีบอัดใช้ interleaved RLE ของโปรโตคอล Remote Desktop
  (MS-RDPBCGR 3.1.9) ซึ่ง Juntei คลายการบีบอัดตามที่ข้อกำหนดนิยามไว้
- ไทล์ `.bmc` ที่เล็กกว่าช่อง 64 × 64 ของตนจะมีส่วนท้ายของไทล์ก่อนหน้าหลงเหลืออยู่ต่อจากตัวมัน Juntei บันทึกแถวที่หลงเหลือเหล่านั้นเป็นภาพแยกต่างหาก (`RemnantImage`) แถวเหล่านี้เป็นของไทล์ที่เก่ากว่า ไม่ใช่ของไทล์ที่อยู่ในช่องนั้น

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file rdp-cache <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find Cache????.bin and bcache*.bmc files
  -f, --file <FILE>      Parse one cache file, whatever its name

Output:
  -o, --output <FILE>       Save one record per tile
  -t, --output-type <TYPE>  csv, json, or jsonl
  -x, --extract <DIR>       Save each tile as a BMP and a PNG collage of each cache file
      --collage-only        With --extract, save only the collages
      --collage-width <N>   Tiles per collage row (default 64)
```

```bash
juntei parse-file rdp-cache -d evidence/ -o rdp-cache.csv --extract rdp-cache/
juntei parse-file rdp-cache -f Cache0000.bin --extract rdp-cache/ --collage-only
juntei parse-file rdp-cache -d evidence/ -t jsonl | jq 'select(.SingleColor == false)'
```

เมื่อใช้ `-d` นั้น Juntei จะอ่านไฟล์ที่มีชื่อเป็น `Cache` ตามด้วยตัวเลขและ `.bin` และไฟล์ที่มีชื่อเป็น `bcache`
ตามด้วยตัวเลขและ `.bmc` ไฟล์ `Cache????.bin` ที่ไม่ได้ขึ้นต้นด้วยลายเซ็น `RDP8bmp` เป็นของโปรแกรมอื่น
ไฟล์ดังกล่าวจะถูกนับไว้ในสรุปและถูกข้ามไป ส่วนเมื่อใช้ `-f` Juntei จะอ่านไฟล์นั้นไม่ว่าจะมีชื่ออะไร

### ภาพ { #images }

`--extract` จะสร้างโฟลเดอร์สำหรับไฟล์แคชแต่ละไฟล์ที่ `<DIR>/<user>/<cache file>/` เมื่อไฟล์แคชสองไฟล์จะต้องใช้โฟลเดอร์เดียวกัน เช่น ไฟล์แคชกับสำเนาของไฟล์นั้นจาก Volume Shadow Copy โฟลเดอร์ที่สองจะมีหมายเลขต่อท้าย (`Cache0001.bin_2`) ภายในโฟลเดอร์มี:

- `<cache file>_<index>.bmp`: ไทล์แต่ละไทล์ตามขนาดจริงของไทล์นั้น ชื่อไฟล์เป็นแบบเดียวกับที่ bmc-tools ใช้
  RdpCacheStitcher จึงเปิดโฟลเดอร์นี้ได้
- `<cache file>_old_<index>.bmp`: เฉพาะ `.bmc` คือส่วนท้ายของไทล์ก่อนหน้าที่หลงเหลืออยู่ต่อจากไทล์ที่เล็กกว่า
- `<cache file>_collage.png`: ทุกไทล์ตามลำดับในไฟล์ แถวละ 64 ไทล์ (`--collage-width`) แต่ละไทล์อยู่ในช่องขนาด 64 × 64 โดยวางชิดมุมซ้ายบนของช่อง ไทล์ที่มีดัชนี *i* อยู่ในแถวที่ *i* / 64 และคอลัมน์ที่ *i* % 64
  โดยนับจาก 0 ส่วนที่ว่างของช่องจะเติมด้วยสีม่วงแดง (magenta) ได้แก่ ข้างไทล์ที่แคบ ใต้ไทล์ที่เตี้ย และตำแหน่งที่อ่านไทล์ไม่ได้

`--collage-only` จะบันทึกเฉพาะภาพคอลลาจ ภาพที่มีอยู่แล้วจะไม่ถูกแทนที่หากไม่ใช้ `-C` และ `Problems` ของไทล์นั้นจะระบุไว้ ส่วนตัวไฟล์แคชเองจะไม่ถูกแทนที่เลย แม้จะใช้ `-C`

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | ความหมาย |
|---|---|
| `SourceFile` | ไฟล์แคช |
| `User` | ผู้ใช้ที่เป็นเจ้าของโฟลเดอร์โปรไฟล์ซึ่งมีไฟล์แคชอยู่ โดยดูจากพาธของไฟล์ |
| `TileIndex` | หมายเลขของไทล์ในไฟล์ นับจาก 0 เหมือนกับในชื่อไฟล์ภาพของไทล์ |
| `Width`, `Height` | ขนาดของไทล์เป็นพิกเซล |
| `SingleColor` | เป็น `true` เมื่อทุกพิกเซลของไทล์มีสีเดียวกัน |
| `TileImage` | เมื่อใช้ `--extract`: ไฟล์ BMP ของไทล์ |
| `Collage` | เมื่อใช้ `--extract`: ภาพคอลลาจของไฟล์แคช |
| `RemnantImage` | `.bmc` เมื่อใช้ `--extract`: ส่วนท้ายของไทล์ก่อนหน้าที่หลงเหลืออยู่ในช่อง |
| `Key` | คีย์ 64 บิตของไทล์ในโปรโตคอล Remote Desktop เป็นเลขฐานสิบหก |
| `Format`, `FormatVersion` | `bin` พร้อมเวอร์ชันที่อยู่ต่อจากลายเซ็น (6 บน Windows 10 ขึ้นไป) หรือ `bmc` |
| `BitsPerPixel` | 32 สำหรับ `.bin`; 8, 16, 24 หรือ 32 สำหรับ `.bmc` |
| `Compressed`, `Flags` | `.bmc`: ไทล์ถูกบีบอัดหรือไม่ และแฟล็กในส่วนหัวของไทล์ |
| `Offset` | ตำแหน่งที่ส่วนหัวของไทล์เริ่มต้นในไฟล์ |
| `Problems` | ความเสียหาย ได้แก่ ไฟล์ที่สิ้นสุดกลางไทล์ ส่วนหัวของไทล์ที่ไม่มีไทล์ใดจะมีได้ ข้อมูลที่ไม่สอดคล้องกับขนาดของไทล์ ข้อมูลที่บีบอัดซึ่งคลายการบีบอัดไม่ได้ และภาพที่บันทึกไม่ได้ |

JSON และ JSONL มีฟิลด์เดียวกัน โดยตัวเลขและค่าบูลีนมีชนิดข้อมูลตามจริง ค่าที่ไทล์ไม่มีจะเป็น `null` และ
`Problems` เป็นรายการ (list) ความเสียหายที่ไม่ได้อยู่ในไทล์ใดไทล์หนึ่ง เช่น ข้อมูลที่อยู่ต่อจากไทล์สุดท้าย
จะรายงานไว้ในข้อความของการทำงานครั้งนั้น

## การวิเคราะห์อัตโนมัติ { #automatic-parsing }

`auto-parse` เขียน `LateralMovement/RdpCache.csv` (`RdpCache.json` หรือ `RdpCache.jsonl` เมื่อใช้
`-t json` หรือ `-t jsonl`) พร้อม `LateralMovement/RdpCache-logs.txt` และบันทึกภาพคอลลาจของไฟล์แคชแต่ละไฟล์ไว้ในโฟลเดอร์ `LateralMovement/RdpCache/` เพิ่ม `--extract-rdp-tiles` เพื่อบันทึกทุกไทล์ด้วย ซึ่งอาจมีจำนวนหลายหมื่นไฟล์สำหรับผู้ใช้คนเดียว เมื่อวิเคราะห์จากดิสก์อิมเมจ ไฟล์แคชจะถูกคัดลอก ไทม์ไลน์แบบรวม `-j`
จะรวมทุกแถว พร้อมตัวระบุ `Artifact` ที่เป็น `rdp-cache` ไทล์ไม่มีเวลา
[`super-timeline`](../commands/super-timeline.md) จึงไม่สร้างเหตุการณ์จากไทล์เหล่านี้

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เปิดภาพคอลลาจและมองหาข้อความ ได้แก่ ชื่อคอมพิวเตอร์และชื่อผู้ใช้ พาธของไฟล์ คำสั่ง หน้าต่างคอนโซล
   แถบที่อยู่ของเบราว์เซอร์ และกล่องโต้ตอบ
2. ประกอบหน้าจอขึ้นใหม่ด้วย RdpCacheStitcher จากโฟลเดอร์ไทล์ที่บันทึกด้วย `--extract`
3. ค้นหาคอมพิวเตอร์ที่ผู้ใช้เชื่อมต่อไป จากรีจิสทรีของไคลเอนต์ (`parse-registry remote-access`) ไฟล์ `.rdp`
   และ Jump Lists ของ `mstsc.exe` แล้วค้นหาการเข้าสู่ระบบผ่าน RDP ที่ตรงกันและเรคคอร์ดการบันทึกการเข้าถึงของผู้ใช้ (UAL) บนคอมพิวเตอร์เหล่านั้น
4. ใช้เวลาใน `$MFT` ของไฟล์แคชและของสำเนาไฟล์แคชใน Volume Shadow Copy เพื่อจำกัดช่วงเวลาที่ไทล์ถูกเขียน

## รายละเอียดเพิ่มเติม { #further-details }

- [ANSSI (CERT-FR) — CERTFR-2016-ACT-017: แคชบิตแมปของ RDP (ภาษาฝรั่งเศส)](https://www.cert.ssi.gouv.fr/actualite/CERTFR-2016-ACT-017/)
- [ANSSI — bmc-tools](https://github.com/ANSSI-FR/bmc-tools)
- [BSI — RdpCacheStitcher](https://github.com/BSI-Bund/RdpCacheStitcher)
- [Velociraptor — Windows.Forensics.RDPCache](https://github.com/Velocidex/velociraptor/blob/master/artifacts/definitions/Windows/Forensics/RDPCache.yaml)
- [dissect.target — ปลั๊กอินแคช RDP](https://github.com/fox-it/dissect.target/blob/main/dissect/target/plugins/os/windows/rdpcache.py)
- [Microsoft — MS-RDPBCGR 3.1.9: Interleaved RLE-Based Bitmap Compression](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-rdpbcgr/b6a3f5c2-0804-4c10-9d25-a321720fd23e)
- [FreeRDP — แคชบิตแมปแบบถาวร](https://github.com/FreeRDP/FreeRDP/blob/master/libfreerdp/cache/persistent.c)
