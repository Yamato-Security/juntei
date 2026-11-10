# การตรวจสอบครั้งแรก { #first-investigation }

ขั้นตอนนี้สมมติว่าคุณได้รับชุด triage ที่เก็บรวบรวมแบบออฟไลน์ไว้ใน `evidence/`

## 1. เก็บรักษาแหล่งข้อมูลต้นทาง { #1-preserve-the-source }

ทำงานจากสำเนาที่ตรวจสอบแล้ว บันทึกแฮช เวลาที่เก็บรวบรวม เขตเวลา โฮสต์ วอลุ่ม และเครื่องมือที่ใช้
เก็บรวบรวม Juntei อ่านหลักฐานเท่านั้น แต่ตัวเลือกบางอย่าง เช่น การดึง resident stream หรือการดึงข้อมูลที่ถูกกักกันออกมา
จะสร้างไฟล์เอาต์พุตใหม่ ให้กำหนดเป้าหมายของไฟล์เหล่านั้นไปยังไดเรกทอรีวิเคราะห์แยกต่างหาก

## 2. สำรวจสิ่งที่คุณมี { #2-inventory-what-you-have }

ค้นหาพาธที่ระบุไว้ใน[การเก็บรวบรวมหลักฐาน](collection.md) การที่อาร์ติแฟกต์บางรายการขาดหายไปเป็นเรื่องปกติ
อย่าสันนิษฐานว่าชุดที่เก็บรวบรวมมี transaction log เพียงเพราะมีไฮฟ์รีจิสทรีอยู่

## 3. เรียกดูสรุปผลก่อน { #3-run-summaries-first }

คำสั่งจำนวนมากจะพิมพ์สรุปผลแบบกระชับเมื่อไม่ได้ระบุไฟล์เอาต์พุต

```bash
juntei parse-file prefetch -d evidence/
juntei parse-registry shimcache -d evidence/
juntei parse-file defender-quarantine -d evidence/
```

อ่านคำเตือนเกี่ยวกับความเสียหาย ฐานข้อมูลที่อยู่ในสถานะ dirty และ log ที่ยังไม่ได้นำมาใช้ ก่อนเชื่อถือเรคคอร์ด

หากต้องการวิเคราะห์อาร์ติแฟกต์ที่รองรับทั้งหมดในชุดที่เก็บรวบรวมจากคอมพิวเตอร์หลายเครื่องด้วยการดำเนินการครั้งเดียว ให้ใช้

```bash
juntei auto-parse -d evidence/ -o analysis/
```

ดู[วิเคราะห์ข้อมูลที่รวบรวมโดยอัตโนมัติ](../commands/auto-parse.md) สำหรับการค้นหาโฮสต์ การทำงานแบบขนาน
โครงสร้างเอาต์พุต และ JSONL แบบรวมสำหรับนำเข้า SIEM

## 4. บันทึกเอาต์พุตแบบมีโครงสร้าง { #4-save-structured-output }

```bash
mkdir -p analysis
juntei parse-file prefetch -d evidence/ -T -o analysis/prefetch.csv
juntei parse-ntfs mft -d evidence/ -o analysis/mft
juntei parse-ntfs usn-journal -d evidence/ -o analysis/usn
juntei parse-registry services -d evidence/ -o analysis/services.jsonl -t jsonl
```

ควรใช้ JSONL สำหรับการเขียนสคริปต์และค่าที่ซ้อนกัน และใช้ CSV สำหรับสเปรดชีตหรือโปรแกรมดูไทม์ไลน์

## 5. เชื่อมโยง (pivot) จากตัวระบุที่เชื่อถือได้ { #5-pivot-from-a-reliable-identifier }

เริ่มต้นด้วยชื่อไฟล์ พาธแบบเต็ม SHA-1/SHA-256, SID ของผู้ใช้, MFT entry/sequence, GUID ของงาน,
ชื่อบริการ หรือช่วงเวลา ให้ทำตัวพิมพ์ให้เป็นมาตรฐานเฉพาะฟิลด์ที่ Windows ถือว่าไม่แยกตัวพิมพ์เล็ก-ใหญ่

```bash
juntei parse-file prefetch -d evidence/ -t jsonl \
  | jq 'select(.ExecutableName? | ascii_downcase == "suspicious.exe")'
```

## 6. เชื่อมโยงหลักฐาน { #6-correlate }

ใช้[การตรวจสอบข้ามอาร์ติแฟกต์](../resources/workflows.md) ลำดับที่ใช้ได้จริงคือ

1. Prefetch สำหรับการเรียกใช้งานและเวลาเรียกใช้ล่าสุด
2. `$MFT` และ USN Journal สำหรับการสร้าง พาธ การเปลี่ยนชื่อ และการลบ
3. Amcache และ ShimCache สำหรับการมีอยู่ในอดีตและเมทาดาทา
4. LNK, Jump Lists, RecentDocs และ SRUM สำหรับบริบทของผู้ใช้/แอปพลิเคชัน
5. บริการในรีจิสทรีและ TaskCache สำหรับการคงอยู่ (persistence)
6. ข้อมูลที่ Defender กักกัน สำหรับการตรวจจับ การนำออก แหล่งที่มา และแฮช

## 7. บันทึกข้อมูลเพื่อให้ทำซ้ำได้ { #7-document-reproducibility }

บันทึกพาธต้นทาง แฮชของต้นทาง เวอร์ชันของ Juntei คำสั่งที่ใช้ทุกประการ แฮชของเอาต์พุต เขตเวลาของระบบ
และการกรองใด ๆ ที่ทำภายหลัง อย่าแก้ไขเอาต์พุตหลักฐานที่สร้างขึ้นโดยไม่บันทึกไว้
