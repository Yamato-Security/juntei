# ประวัติการตรวจจับของ Microsoft Defender { #microsoft-defender-detection-history }

## สิ่งที่ประวัติบันทึกไว้ { #what-the-history-records }

Microsoft Defender เขียนไฟล์ประวัติการตรวจจับหนึ่งไฟล์สำหรับการตรวจจับแต่ละครั้ง แต่ละไฟล์บันทึก:

- ตัวระบุ ชื่อ ระดับความรุนแรง และหมวดหมู่ของภัยคุกคาม
- สิ่งที่ Defender ดำเนินการ และดำเนินการสำเร็จหรือไม่
- ภัยคุกคามกำลังทำงานอยู่หรือไม่
- ผู้ใช้และโปรเซสที่เกี่ยวข้อง และสิ่งที่ตรวจจับภัยคุกคามนั้นได้
- entry การกักกัน หากมี
- ทุกทรัพยากรที่พบภัยคุกคาม ได้แก่ ไฟล์ สมาชิกในไฟล์บีบอัด (archive) การดาวน์โหลด โปรเซส ค่ารีจิสทรี บริการ บรรทัดคำสั่ง และพฤติกรรม

Defender ยังเขียนไฟล์เหล่านี้สำหรับภัยคุกคามที่ไม่เคยถูกกักกันด้วย ได้แก่ ภัยคุกคามที่เพียงตรวจจับได้ ภัยคุกคามที่ได้รับอนุญาต และภัยคุกคามที่ลบไม่สำเร็จ ประวัติการป้องกันของ Windows Security และคลาส `MSFT_MpThreatDetection` ของ WMI (`Get-MpThreatDetection`) แสดงระเบียนเหล่านี้ เมื่อ Orange Cyberdefense เปลี่ยนค่าในไฟล์ WMI ก็รายงานค่าที่ถูกเปลี่ยนแล้ว

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> Defender ได้บันทึกการตรวจจับภัยคุกคามที่ระบุชื่อในทรัพยากรที่แสดงไว้ พร้อมสถานะและเวลาที่บันทึกไว้ ชื่อภัยคุกคามเป็นคำตัดสินของผลิตภัณฑ์ ระเบียนการตรวจจับเพียงอย่างเดียวไม่ได้แสดงว่าภัยคุกคามเคยทำงาน หรือถูกลบออกไปอย่างสมบูรณ์

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\ProgramData\Microsoft\Windows Defender\Scans\History\Service\DetectionHistory\<nn>\<GUID>
```

แต่ละไฟล์ตั้งชื่อตาม `DetectionID` ของการตรวจจับนั้น ให้เก็บรวบรวมโฟลเดอร์ `DetectionHistory` ทั้งหมด อินพุตแบบไดเรกทอรีและ `auto-parse` จะอ่านทุกไฟล์ภายในโฟลเดอร์ `DetectionHistory` ที่ขึ้นต้นเหมือนไฟล์ประวัติการตรวจจับ ไม่ว่าจะอยู่ลึกเพียงใด ใช้ `-f` สำหรับสำเนาที่เก็บไว้นอกโฟลเดอร์ดังกล่าว Defender จะลบรายการออกจากประวัติการสแกนหลังจากผ่านไปตามจำนวนวันที่กำหนดค่าได้ (`ScanPurgeItemsAfterDelay` ใน `Set-MpPreference`) การตรวจจับที่เก่ากว่าจึงอาจหายไปแล้ว

พื้นที่กักกันเป็นอาร์ติแฟกต์แยกต่างหาก ดู [ข้อมูลที่ Defender กักกัน](defender-quarantine.md) `QuarantineID` ของการตรวจจับใช้การสะกดแบบเดียวกับ `EntryID` ในเอาต์พุตการกักกัน (ตัวพิมพ์ใหญ่ ไม่มีวงเล็บปีกกา) จึงสามารถเชื่อม (join) ทั้งสองเข้าด้วยกันได้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- การตรวจจับ ความพยายามแก้ไข และการลบสำเร็จเป็นสถานะที่แยกจากกัน ให้อ่าน `ThreatStatus` (`Detected`, `Quarantined`, `Removed`, `Allowed`, `RemoveFailed`, …) ร่วมกับ `ThreatStatusErrorCode` ซึ่งเป็น 0 เมื่อการเปลี่ยนสถานะครั้งล่าสุดสำเร็จ
- `ExecutionStatus` คือมุมมองของ Defender ณ เวลาที่ตรวจจับ (`Blocked`, `Allowed`, `Executing`, `NotExecuting`, `Unknown`) ไม่ใช่ข้อพิสูจน์ว่าภัยคุกคามเคยทำงาน และไม่ใช่ข้อพิสูจน์ว่าไม่เคยทำงาน ให้ยืนยันร่วมกับ Prefetch, Amcache, event log และข้อมูลเทเลเมทรีของโปรเซส
- `ThreatName` คือชื่อ signature ณ เวลานั้น อาจเป็นผลบวกลวง (false positive) หรือถูกจัดประเภทใหม่ในภายหลังได้ ไฟล์ทดสอบ เช่น EICAR จะถูกรายงานเหมือนภัยคุกคามจริง
- `User` คือค่า `DomainUser` ที่ Defender บันทึกไว้ (`UserSource` เป็น `DomainUser`) อาจว่างเปล่า เป็นบัญชีบริการ หรือเป็นผู้ใช้คนละคนกับผู้ที่สร้างไฟล์
- Defender อาจเขียนหลายไฟล์สำหรับภัยคุกคามเดียว และ Juntei เก็บไว้ทุกไฟล์ สองไฟล์อาจมี `DetectionID` เดียวกันด้วยซ้ำ เช่น ไฟล์ที่ถูกคัดลอกหรือกู้คืน ให้จัดกลุ่มตาม `DetectionID` และ `SourceFile` ร่วมกัน
- ค่าที่ไม่ทราบความหมายจะถูกเก็บไว้โดยไม่ตีความ ส่วนและตำแหน่งของค่าเหล่านั้นอยู่ใน `UnknownValues`

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-file defender-history <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find detection history files
  -f, --file <FILE>      Parse one DetectionHistory\<nn>\<GUID> file

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
  -C, --clobber             Overwrite an existing output
```

```bash
juntei parse-file defender-history -d evidence/ -o defender-history.csv
juntei parse-file defender-history -d evidence/ -t jsonl \
  | jq '{InitialDetectionTime, ThreatName, ThreatStatus, ResourceType, Path, SHA256}'
```

CSV เป็นรูปแบบไฟล์เริ่มต้น เมื่อใช้ `-t csv`, `-t json` หรือ `-t jsonl` โดยไม่มี `-o` จะเขียนระเบียนไปยัง stdout หากไม่มีทั้งสองตัวเลือกจะแสดงเพียงสรุป เพิ่ม `-C` เพื่อเขียนทับเอาต์พุตที่มีอยู่ เอาต์พุตจะไม่ถูกเขียนทับไฟล์อินพุตใด ๆ เลย แม้จะใช้ `-C`

Juntei เขียนหนึ่งระเบียนต่อหนึ่งทรัพยากรของการตรวจจับแต่ละครั้ง การตรวจจับที่ไม่มีทรัพยากรจะได้หนึ่งระเบียนที่ฟิลด์ทรัพยากรว่างเปล่า ระเบียนของการตรวจจับเดียวกันใช้ `DetectionID` ร่วมกัน และมีหมายเลข `ResourceIndex` ตามลำดับที่จัดเก็บ

ความเสียหายไม่ทำให้การทำงานหยุดลง ไฟล์ที่ถูกตัดสั้นจะเก็บค่าที่อ่านได้ก่อนถึงส่วนที่เสียหาย และ `Problems` จะบอกว่าขาดอะไรไป ไฟล์ในโฟลเดอร์ `DetectionHistory` ที่ไม่ใช่ไฟล์ประวัติการตรวจจับจะถูกรายงานและข้ามไป ไฟล์อื่นยังคงถูกอ่านตามปกติ

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Record` | `resource` หรือ `detection` สำหรับการตรวจจับที่ไม่มีทรัพยากร |
| `InitialDetectionTime` | เวลาที่ Defender ตรวจจับภัยคุกคามได้ครั้งแรก (UTC) ว่างเปล่าเมื่อไฟล์เก็บค่า 0 ค่าที่ไม่ใช่เวลาจะถูกเก็บไว้ใน `UnknownValues` |
| `LastThreatStatusChangeTime` | เวลาที่สถานะของภัยคุกคามเปลี่ยนครั้งล่าสุด (UTC) |
| `RemediationTime` | เวลาที่ Defender แก้ไขภัยคุกคาม (UTC) เป็นเวลาของความพยายามแก้ไข ไม่ใช่ข้อพิสูจน์ว่าลบออกแล้ว ให้ตรวจสอบ `ThreatStatus` |
| `ThreatName`, `ThreatID` | ชื่อ signature และตัวระบุที่เป็นตัวเลขของ Defender (`Get-MpThreatCatalog`) |
| `Severity`, `SeverityID` | `Low`, `Moderate`, `High`, `Severe` หรือ `Unknown` พร้อมตัวเลขที่จัดเก็บไว้ |
| `Category`, `CategoryID` | หมวดหมู่ภัยคุกคาม เช่น `TROJAN`, `TOOL` หรือ `BEHAVIOR` พร้อมตัวเลขที่จัดเก็บไว้ |
| `ThreatStatus`, `ThreatStatusID` | สถานะของภัยคุกคามตามที่ Defender กำหนด ตัวเลขที่ไม่มีชื่อในเอกสารจะถูกเก็บไว้โดยให้ `ThreatStatus` ว่างเปล่า |
| `ThreatStatusErrorCode` | รหัสข้อผิดพลาดของการเปลี่ยนสถานะครั้งล่าสุด 0 หมายถึงสำเร็จ |
| `ExecutionStatus`, `ExecutionStatusID` | Defender เห็นว่าภัยคุกคามกำลังทำงานอยู่หรือไม่ในขณะที่ตรวจจับได้ |
| `DetectionSource`, `DetectionSourceTypeID` | สิ่งที่ตรวจจับ ได้แก่ `Real-time`, `IOAV` (การดาวน์โหลดและไฟล์แนบ), `User`, `System`, `AMSI` และอื่น ๆ |
| `User`, `UserSource` | ค่า `DomainUser` ที่ Defender บันทึกไว้ และที่มาของค่า (`DomainUser`) ว่างเปล่าเมื่อไฟล์ไม่ได้ระบุผู้ใช้ |
| `Process` | โปรเซสที่เกี่ยวข้อง (`ProcessName`) อาจเป็น `Unknown` |
| `ResourceIndex`, `ResourceCount` | ตำแหน่งของทรัพยากรนี้ และจำนวนทรัพยากรในการตรวจจับ |
| `ResourceType` | `file`, `containerfile`, `webfile`, `process`, `regkeyvalue`, `service`, `CmdLine`, `behavior`, `amsi` และอื่น ๆ ตามชื่อที่ Defender ใช้ |
| `Path` | ตัวทรัพยากร ได้แก่ พาธ สมาชิกในไฟล์บีบอัด (`archive.zip->member`) ค่ารีจิสทรี บรรทัดคำสั่ง หรือโปรเซส `webfile` ยังระบุ URL และโปรเซสที่ดาวน์โหลด (`pid:…,ProcessStart:…`) ด้วย ให้เก็บรักษาการสะกดไว้ตรงตามเดิม |
| `ResourceFlags` | แฟล็กของทรัพยากรในรูปเลขฐานสิบหก ไม่มีเอกสารระบุความหมาย |
| `TrackingStartTime` | `ThreatTrackingStartTime` ของทรัพยากร ซึ่งเป็นจำนวนเต็มที่อ่านเป็น FILETIME (UTC) แบบเดียวกับที่ plaso ใช้ระบุวันที่ของการตรวจจับ |
| `SHA256`, `SHA1`, `MD5`, `TrackingSize` | แฮชและขนาดจากข้อมูลการติดตามของทรัพยากร ซึ่งอธิบายไฟล์ ณ เวลาที่ Defender สแกน |
| `TrackingID` | `ThreatTrackingId` ของทรัพยากร |
| `OtherTracking` | ค่าการติดตามอื่นทั้งหมด ในรูป `Key=Value` ตามลำดับที่จัดเก็บ เช่น `ThreatTrackingScanSource` หรือ `ThreatTrackingBMContextRich` สำหรับการตรวจจับพฤติกรรม |
| `QuarantineID` | entry การกักกัน (`Quarantine\Entries\{GUID}`) ว่างเปล่าเมื่อไฟล์เก็บ GUID ที่เป็นศูนย์ทั้งหมด |
| `DetectionID` | GUID ของการตรวจจับ และเป็นชื่อไฟล์ด้วย หากไม่ตรงกันจะรายงานใน `Problems` |
| `MagicVersion` | `Magic.Version:1.2` ของส่วนหัว ว่างเปล่าในไฟล์ที่เขียนโดยไม่มีค่านี้ เช่น บางไฟล์จาก Windows Server 2016 |
| `UnknownValues` | ค่าที่ไม่ทราบความหมาย ในรูป `position=value` (เช่น `6.13=NT AUTHORITY\SYSTEM`) โดยรายการแบบนับจำนวนที่อยู่หลังสถานะจะอยู่ในรูป `2.list` |
| `SourceFile` | ไฟล์ที่อ่าน เป็นพาธสัมพัทธ์จากไดเรกทอรีอินพุต หรือชื่อไฟล์เมื่อใช้ `-f` |
| `Problems` | ความเสียหาย ค่าที่ขาดหาย ชนิดข้อมูลที่ไม่คาดคิด หรือชื่อไฟล์ที่ไม่ตรงกัน ห้ามซ่อนไว้ |

JSON และ JSONL มีฟิลด์เดียวกันโดยค่ามีชนิดข้อมูล ตัวเลขยังคงเป็นตัวเลข และค่าที่ขาดหายเป็น `null` แทนที่จะมี `OtherTracking` จะมี `Tracking` ซึ่งเป็นค่าการติดตามทุกค่าพร้อมคีย์ของ Defender ชนิด และค่า `UnknownValues` จะกลายเป็นรายการตำแหน่งพร้อมชนิด `CountedList` เก็บรายการแบบนับจำนวน FILETIME ที่จัดเก็บไว้จะเก็บจำนวน tick ดิบไว้ข้างเวลาที่อ่านได้

## พฤติกรรมในไทม์ไลน์ { #timeline-behavior }

`auto-parse` บันทึกระเบียนเป็น `SecurityProducts/DefenderHistory.csv` (หรือ `.json`/`.jsonl`) และเพิ่มลงในไทม์ไลน์แบบรวม `-j` ส่วน `super-timeline` สร้างเหตุการณ์จาก `InitialDetectionTime`, `LastThreatStatusChangeTime` และ `RemediationTime` บนระเบียนของแต่ละทรัพยากร พร้อมใส่สถานะ รหัสข้อผิดพลาด สถานะการทำงาน และแหล่งที่ตรวจจับไว้ในรายละเอียดของเหตุการณ์ `TrackingStartTime` ถูกจัดเก็บเป็นจำนวนเต็ม จึงจะกลายเป็นเหตุการณ์เฉพาะเมื่อใช้ `--include-interpreted-times` ไม่มีการใช้เวลาของตัวไฟล์เองเลย ระเบียนที่ไม่มีเวลาที่ใช้ได้จะยังอยู่ในเอาต์พุตที่วิเคราะห์แล้ว และนับเป็นระเบียนที่ไม่มีเวลา

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เริ่มจาก `ThreatName`, `Path`, `SHA256`, `InitialDetectionTime` และ `ThreatStatus`
2. ตรวจสอบ `ThreatStatusErrorCode` และ `Problems` ก่อนสรุปว่าภัยคุกคามถูกลบออกแล้ว
3. เชื่อม `QuarantineID` กับ `EntryID` ของการกักกัน เพื่อกู้คืนไบต์ที่ถูกกักกัน
4. ใช้แฮชและพาธเชื่อมโยงไปยัง `$MFT`, บันทึก USN, Prefetch, Amcache, ประวัติเบราว์เซอร์ และ event log (operational log ของ Defender: event ID 1116, 1117, 1118 และ 1119)
5. สำหรับทรัพยากร `webfile` ให้ใช้ URL และเวลาเริ่มโปรเซสเพื่อตรวจสอบการเข้าถึงครั้งแรก (initial access) และยืนยันด้วยหลักฐานจากเบราว์เซอร์ พร็อกซี และ DNS

## เวอร์ชันที่ทดสอบ { #tested-versions }

การทดสอบใช้ไฟล์สาธารณะที่มีผลลัพธ์ซึ่งเครื่องมืออื่นอ่านได้อย่างอิสระ:

- ตัวอย่าง Windows 10 ของ Orange Cyberdefense (Defender 4.18.24090.11) ตรวจสอบเทียบกับค่าที่ WMI รายงาน
- ไฟล์ทดสอบ Windows สองไฟล์ของ plaso ตรวจสอบเทียบกับพาร์เซอร์ `windefender_history` ของ plaso
- 14 ไฟล์จาก VM จำลอง APT ของ DFIRArtifactMuseum
- สำเนาที่เขียนโดยไม่มีเวอร์ชันในส่วนหัว เช่นเดียวกับบางไฟล์จาก Windows Server 2016
- ไฟล์ที่ถูกตัดสั้น ไฟล์ว่าง หรือไฟล์ชนิดอื่น

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-defender-history-parser](https://github.com/Yamato-Security/ys-defender-history-parser)
- [Sootmark defender](https://github.com/Sootmark/defender)
- [Orange Cyberdefense wddh-parser](https://github.com/cert-orangecyberdefense/wddh-parser)
- [พาร์เซอร์ประวัติ Defender ของ plaso](https://github.com/log2timeline/plaso/blob/main/plaso/parsers/windefender_history.py)
- [Velociraptor Windows.Applications.DefenderHistory](https://github.com/Velocidex/velociraptor-docs/blob/master/content/exchange/artifacts/Windows.Applications.DefenderHistory.yaml)
- [Microsoft: คลาส MSFT_MpThreatDetection](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/defender/msft-mpthreatdetection)
- [Microsoft: event ID ของ Microsoft Defender Antivirus](https://learn.microsoft.com/en-us/defender-endpoint/troubleshoot-microsoft-defender-antivirus)
