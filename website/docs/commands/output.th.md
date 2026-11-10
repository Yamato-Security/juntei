# วิธีส่งออกข้อมูลทั่วไป { #common-output-patterns }

## อาร์ติแฟกต์เดียวหรือทั้งชุดที่เก็บรวบรวม { #one-artifact-or-a-collection }

```bash
juntei parse-file prefetch -f 'Prefetch/POWERSHELL.EXE-12345678.pf'
juntei parse-file prefetch -d evidence/
```

`-d` ค้นหาไฟล์ที่เหมาะกับคำสั่งนั้นแบบเรียกซ้ำ คอลัมน์พาธต้นทางช่วยแยกแยะ
โฮสต์และอาร์ติแฟกต์ในเอาต์พุตที่รวมกัน

## บันทึกเป็น CSV { #save-csv }

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv
```

พาร์เซอร์บางตัวต้องการไดเรกทอรี เนื่องจากส่งออกตาราง CSV หลายตาราง

```bash
juntei parse-registry amcache -f Amcache.hve -o amcache-tables
juntei parse-ese srum -f SRUDB.dat -o srum-tables
juntei parse-file jump-lists -d evidence/ -o jump-list-tables
```

## สตรีม JSONL { #stream-jsonl }

```bash
juntei parse-registry services -d evidence/ -t jsonl \
  | jq -c 'select(.ImagePath? | test("temp|public"; "i"))'
```

ข้อความและความคืบหน้าจะส่งไปยัง standard error ส่วน standard output ยังคงนำไปแยกวิเคราะห์ต่อได้

## เขียนทับโดยเจตนา { #overwrite-intentionally }

โดยค่าเริ่มต้น Juntei จะไม่ยอมเขียนทับเอาต์พุตที่มีอยู่แล้ว ให้ยืนยันเป้าหมาย เก็บรักษาผลลัพธ์ก่อนหน้า
ที่สำคัญไว้ แล้วจึงเพิ่ม `-C` / `--clobber`

```bash
juntei parse-file prefetch -d evidence/ -o prefetch.csv --clobber
```

## โหมดเงียบและโหมดไม่มีสี { #quiet-and-no-color-modes }

ใช้ `-q` สำหรับงานอัตโนมัติ และ `-K` เมื่อรหัสสีของเทอร์มินัลไม่เหมาะสม

```bash
juntei parse-ntfs mft -f '$MFT' -o mft.csv -q -K
```

## ตัวพิมพ์และพาธ { #case-and-paths }

ใส่เครื่องหมายคำพูดครอบพาธ Windows ที่มีช่องว่าง `$` อักขระพิเศษของ shell หรือแบ็กสแลชตามที่ shell
ของคุณต้องการ บน shell แบบ Bash เครื่องหมายคำพูดเดี่ยวจะป้องกันไม่ให้ `$MFT` และ `$J` กลายเป็นตัวแปร

```bash
juntei parse-ntfs mft -f '/evidence/C/$MFT' -o mft.csv
```
