# ไฮฟ์รีจิสทรีของ Windows { #windows-registry-hives }

## ไฮฟ์รีจิสทรีคืออะไร { #what-registry-hives-are }

Windows Registry จัดเก็บการกำหนดค่าของระบบและผู้ใช้ไว้ในไฟล์ไฮฟ์แบบไบนารี คีย์ประกอบกันเป็นโครงสร้างต้นไม้
ส่วนค่า (value) ภายใต้คีย์จะเก็บข้อมูลที่มีชนิดกำกับ อาร์ติแฟกต์รีจิสทรีอธิบายการคงอยู่ (persistence) ซอฟต์แวร์ที่ติดตั้ง
บริการ บัญชี การกำหนดค่าเครือข่าย กิจกรรมของผู้ใช้ การตั้งค่าความปลอดภัย และสถานะของ Windows
แทบทุกส่วน

ตัวอ่านทั่วไป (generic reader) ของ Juntei ถอดรหัสไฮฟ์หลักได้ทุกไฮฟ์ เล่นซ้ำบันทึกธุรกรรม รายงานความเสียหาย และสามารถ
กู้คืนเรคคอร์ดที่ถูกลบ/อยู่ใน free cell, cell slack, ข้อมูลตกค้าง (remnant), สถานะก่อนการเล่นซ้ำ และเรคคอร์ดที่มีอยู่เฉพาะในบันทึกธุรกรรม

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ไฮฟ์มีหรือเก็บรักษาสถานะของคีย์/ค่าไว้ในรูปแบบ
ที่กู้คืนได้ เวลาเขียนล่าสุดของคีย์แสดงว่าคีย์มีการเปลี่ยนแปลง ไม่ได้บอกว่าค่าใดเปลี่ยนหรือเปลี่ยนเพราะเหตุใด

## ไฮฟ์ทั่วไปและตำแหน่ง { #common-hives-and-locations }

| ไฮฟ์ | พาธทั่วไป | ตัวอย่างหลักฐาน |
|---|---|---|
| `SYSTEM` | `C:\Windows\System32\config\SYSTEM` | บริการ ไดรเวอร์ control set ชื่อโฮสต์ อุปกรณ์ที่เมานต์ ShimCache |
| `SOFTWARE` | `C:\Windows\System32\config\SOFTWARE` | ซอฟต์แวร์ที่ติดตั้ง TaskCache โปรไฟล์ การกำหนดค่า Windows |
| `SAM` | `C:\Windows\System32\config\SAM` | บัญชีภายในเครื่องและเมทาดาทาของบัญชี |
| `SECURITY` | `C:\Windows\System32\config\SECURITY` | นโยบายความปลอดภัยภายในเครื่อง/เมทาดาทาของข้อมูลลับ (ขึ้นอยู่กับสิทธิ์การเข้าถึง) |
| `DEFAULT` | `C:\Windows\System32\config\DEFAULT` | การกำหนดค่าโปรไฟล์ระบบเริ่มต้น |
| `NTUSER.DAT` | `C:\Users\<user>\NTUSER.DAT` | การตั้งค่าของผู้ใช้ Run key, RecentDocs, กิจกรรมของแอปพลิเคชัน |
| `UsrClass.dat` | `%LOCALAPPDATA%\Microsoft\Windows\UsrClass.dat` | shell class, กิจกรรม COM/shell ของผู้ใช้ |
| `Amcache.hve` | `C:\Windows\AppCompat\Programs\Amcache.hve` | รายการแอปพลิเคชัน (ใช้ `parse-registry amcache` เพื่อให้ได้เอาต์พุตเชิงความหมาย) |

ให้เก็บรวบรวมไฟล์ `.LOG`, `.LOG1` และ `.LOG2` ที่อยู่ข้างกันด้วย ไฮฟ์ที่ถูกคัดลอกขณะ Windows ทำงานอยู่อาจอยู่ในสถานะ
dirty บันทึกธุรกรรมมีสถานะที่ commit แล้วซึ่งไฟล์หลักเพียงอย่างเดียวยังไม่สะท้อน

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- timestamp การเขียนล่าสุดของคีย์ใช้กับ **คีย์** ไม่ใช่ค่าแต่ละค่า
- โดยปกติค่าในรีจิสทรีไม่มี timestamp ของตนเอง
- `CurrentControlSet` เป็นชื่อแทน (alias) ของนักวิเคราะห์/Windows ที่ resolve ผ่าน `SYSTEM\Select` ไม่ใช่
  คีย์ที่จัดเก็บอยู่จริง
- เรคคอร์ดที่ถูกลบอาจไม่สมบูรณ์ ล้าสมัย ซ้ำกันข้ามแหล่งการกู้คืน หรือถูกเขียนทับ
- เรคคอร์ดที่มีเฉพาะในบันทึกธุรกรรมแสดงสถานะในอดีต/สถานะระหว่างกลาง ให้อธิบาย `recovered_from` ด้วย
- ผลลัพธ์ `absent_value` หมายความว่าคีย์ที่สอบถามมีอยู่ แต่ไม่มีค่านั้นในสถานะที่พาร์สได้
  ไม่ได้พิสูจน์ว่าค่านั้นไม่เคยมีอยู่
- สตริง พาธ และบรรทัดคำสั่งเป็นการกำหนดค่า ไม่ใช่หลักฐานว่าการกระทำที่กำหนดไว้ได้ทำงานจริง
- ไฮฟ์ที่อยู่ในสถานะ dirty โดยไม่มีบันทึกที่นำมาใช้ได้ อาจละเว้นหรือแสดงสถานะที่ล้าสมัย

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry read <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find primary registry hives
  -f, --file <FILE>      Parse one hive
      --log1 <FILE>      Supply first transaction log explicitly
      --log2 <FILE>      Supply second transaction log explicitly
      --no-logs          Intentionally read the primary file without replay

Filtering:
  -k, --key <PATH>             Exact key (case-insensitive)
      --key-regex <REGEX>      Key-path regular expression
      --value <NAME>           Exact value name ("" is default)
      --value-regex <REGEX>    Value-name regular expression
      --data-regex <REGEX>     Rendered-data regular expression
      --timeline-start <DATE>  Key last-write lower bound
      --timeline-end <DATE>    Key last-write upper bound
      --recover-deleted        Recover deleted and transaction-history records

Output:
  -o, --output <FILE>       Save CSV/JSON/JSONL
  -t, --output-type <TYPE>  csv, json, or jsonl
      --raw-data            Include exact bytes for every value
      --qword-as-filetime   Also render REG_QWORD as FILETIME
```

```bash
# Inspect one persistence key
juntei parse-registry read -f NTUSER.DAT \
  -k 'Software\Microsoft\Windows\CurrentVersion\Run' -o run.csv

# Search allocated and recoverable values for a path/name
juntei parse-registry read -f SYSTEM --recover-deleted \
  --data-regex 'powershell|cmd\.exe|users\\public' -t jsonl

# Show raw and decoded data for forensic validation
juntei parse-registry read -f SOFTWARE --raw-data -o software.jsonl -t jsonl
```

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host` | ชื่อโฮสต์ที่ได้จากบริบทของระบบที่มีอยู่ ว่างเมื่อ Juntei ไม่สามารถระบุได้อย่างปลอดภัย |
| `HiveType` | ประเภทไฮฟ์ที่ตรวจพบ (`SYSTEM`, `SOFTWARE`, `NTUSER.DAT` ฯลฯ) ใช้ในการตีความพาธ |
| `SourcePath` | ไฮฟ์อินพุตที่แน่นอน จำเป็นเมื่อโหมดไดเรกทอรีรวมหลายโฮสต์/ผู้ใช้เข้าด้วยกัน |
| `RecordType` | `key`, `value`, `default_value` หรือ `absent_value` การไม่มีอยู่ใช้กับการสอบถาม/สถานะนั้นเท่านั้น |
| `KeyPath` | พาธภายใต้ราก (root) ของไฮฟ์ `[unknown]` ระบุลำดับบรรพบุรุษของรายการที่ถูกลบซึ่ง resolve ไม่ได้ |
| `KeyLastWrittenUTC` | การแก้ไขครั้งล่าสุดของคีย์โดยรวม ไม่สามารถระบุได้ว่าค่าใดเปลี่ยน |
| `ValueName` | ชื่อของค่า ว่างสำหรับค่าเริ่มต้น/ค่าที่ไม่มีชื่อ |
| `ValueType` | ชนิดของรีจิสทรี (`REG_SZ`, `REG_DWORD`, `REG_BINARY` ฯลฯ) ชนิดมีผลต่อการตีความ |
| `ValueData` | ค่าที่แสดงผลตามชนิด multi-string จะยังเป็นลิสต์ใน JSON |
| `DataSize` | ขนาดที่ค่าบันทึกไว้ ให้เปรียบเทียบกับไบต์ดิบและ `Problems` |
| `ValueDataRaw` | ไบต์ที่แน่นอนเมื่อการถอดรหัสล้มเหลวหรือเมื่อใช้ `--raw-data` ให้เก็บรักษาไว้เพื่อการตรวจสอบความถูกต้อง |
| `ValueU64` | ค่าตัวเลขเต็มของ DWORD/QWORD โดยไม่สูญเสียความแม่นยำแบบ JavaScript |
| `ValueFiletimeUTC` | การแสดง QWORD เป็น FILETIME (ไม่บังคับ) มีความหมายเฉพาะเมื่อ schema ของคีย์กำหนดให้เป็น FILETIME |
| `AllocationStatus` | `allocated`, `deleted_complete`, `deleted_partial`, `residual_value_only` หรือ `transaction_log_recovered` เป็นตัวกำหนดระดับความเชื่อมั่น |
| `RecoveredFrom` | `free_cell`, `cell_slack`, `remnant_data`, `before_replay` หรือรายการในบันทึก เช่น `SYSTEM.LOG1#1234` |
| `TransactionLogStatus` | `clean`, `replayed`, `unchanged` หรือ `not_applied` สำหรับบริบทของเรคคอร์ด/ไฮฟ์นี้ |
| `LogsApplied` | ชื่อไฟล์บันทึกธุรกรรมที่ Juntei เล่นซ้ำ ให้บันทึกไว้ในรายงาน |
| `Problems` | ความเสียหาย cell ที่หายไป ชนิดไม่ตรงกัน ลิสต์/parent ที่ resolve ไม่ได้ หรือข้อจำกัดอื่น ๆ |

## การตีความสถานะการกู้คืน { #recovery-status-interpretation }

| สถานะ | ความหมาย |
|---|---|
| `deleted_complete` | คีย์/ค่าที่ถูกลบถูกกู้คืนพร้อมพาธ/ข้อมูลที่ทราบครบถ้วน แต่ยังคงเป็นข้อมูลในอดีต ไม่ใช่สถานะที่ใช้งานอยู่ |
| `deleted_partial` | พาธ ลิสต์ค่า ค่า หรือข้อมูลบางส่วนถูกเขียนทับ/อ่านไม่ได้ อย่าเติมช่องว่างจาก cell ที่ถูกนำกลับมาใช้ใหม่ |
| `residual_value_only` | ค่าหลงเหลืออยู่โดยไม่มีบริบทของคีย์เพียงพอ `KeyPath` อาจไม่มี |
| `transaction_log_recovered` | สถานะมีอยู่เฉพาะในเวอร์ชันก่อนการเล่นซ้ำ/ในบันทึก และไม่มีในเรคคอร์ด allocated/free ปัจจุบัน |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เริ่มต้นด้วยคีย์เชิงความหมายที่แน่นอนเมื่อทราบ ใช้การค้นหาด้วย regex/ข้อมูลเพื่อค้นพบสิ่งใหม่
2. ตรวจสอบ `TransactionLogStatus`, `AllocationStatus`, `RecoveredFrom` และ `Problems` ก่อนดูเนื้อหา
3. ตีความข้อมูลตาม schema ที่มีเอกสารของคีย์ `REG_BINARY` ไม่ได้อธิบายตัวเอง
4. เชื่อมโยงการกำหนดค่ากับหลักฐานขณะทำงาน (event log, Prefetch, SRUM, บริการ/งานตามกำหนดเวลา)
5. เก็บรักษาข้อมูลดิบไว้สำหรับฟิลด์ใหม่หรือฟิลด์ที่มีข้อโต้แย้ง

## รายละเอียดเพิ่มเติม { #further-details }

- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
- [ข้อกำหนดรูปแบบรีจิสทรีโดย Maxim Suhanov](https://github.com/msuhanov/regf)
- [พาร์เซอร์รีจิสทรี yarp](https://github.com/msuhanov/yarp)
- [libyal libregf](https://github.com/libyal/libregf)
- [Registry โดย Eric Zimmerman](https://github.com/EricZimmerman/Registry)
- [Microsoft: ชนิดของค่าในรีจิสทรี](https://learn.microsoft.com/en-us/windows/win32/sysinfo/registry-value-types)
