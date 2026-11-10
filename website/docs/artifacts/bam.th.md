# BAM / DAM { #bam-and-dam }

## BAM และ DAM คืออะไร { #what-bam-and-dam-are }

Background Activity Moderator (BAM) และ Desktop Activity Moderator (DAM) เป็นบริการ Windows
(Windows 10 1709 และใหม่กว่า) ที่ควบคุมว่าแอปพลิเคชันสามารถทำอะไรได้บ้างในเบื้องหลัง
บริการเหล่านี้เก็บเวลาล่าสุดที่แต่ละโปรแกรมทำงานแยกตามผู้ใช้ไว้ในไฮฟ์ `SYSTEM`:

- หนึ่งคีย์ต่อผู้ใช้ ตั้งชื่อตาม SID ของผู้ใช้
- หนึ่งค่าต่อโปรแกรม ชื่อของค่าคือพาธของโปรแกรมบนอุปกรณ์
  (`\Device\HarddiskVolume3\Windows\System32\cmd.exe`) หรือ ID ของแอปแบบแพ็กเกจ
  (`Microsoft.WindowsCalculator_8wekyb3d8bbwe`)
- 8 ไบต์แรกของข้อมูลในค่าคือเวลาที่โปรแกรมทำงานครั้งล่าสุด (FILETIME)

BAM เชื่อมโยงโปรแกรมกับผู้ใช้และเวลาได้โดยไม่ต้องใช้ไฮฟ์ของผู้ใช้เอง และแสดงรายการโปรแกรมที่เรียกใช้
จากอุปกรณ์แบบถอดได้และอุปกรณ์เครือข่าย

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> โพรเซสของโปรแกรมนี้ทำงานในเซสชันของ
ผู้ใช้รายนี้ และทำงานครั้งล่าสุด ณ เวลานี้ BAM ไม่นับจำนวนครั้งที่ทำงาน และรายการอาจยังคงอยู่หลังจาก
โปรแกรมถูกลบไปแล้ว

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SYSTEM
ControlSet00N\Services\bam\State\UserSettings\<SID>   Windows 10 1809 and later
ControlSet00N\Services\bam\UserSettings\<SID>         Windows 10 1709 to 1803
ControlSet00N\Services\dam\State\UserSettings\<SID>
ControlSet00N\Services\dam\UserSettings\<SID>
```

คีย์ของผู้ใช้แต่ละคนยังมีค่า `Version` และ `SequenceNumber` ซึ่งไม่ใช่โปรแกรมจึงถูก
ละไว้ ระบบที่อัปเกรดจาก 1709 หรือ 1803 อาจมีโครงสร้างทั้งสองแบบ

ให้เก็บรวบรวม `SYSTEM` พร้อม transaction log และ `SAM` กับ `SOFTWARE` ของระบบ ซึ่งใช้ระบุชื่อ
ผู้ใช้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **หนึ่งเวลาต่อโปรแกรมและผู้ใช้** เก็บไว้เฉพาะการทำงานครั้งล่าสุดเท่านั้น ส่วนการทำงานครั้งก่อน ๆ ให้ดูจาก Prefetch, Amcache และ
  event log
- **กิจกรรมล่าสุด** มีรายงานว่ารายการของโปรแกรมที่ไม่ได้ทำงานมาระยะหนึ่งจะถูก
  ลบออกเมื่อ Windows เริ่มทำงาน ดังนั้น BAM ส่วนใหญ่จึงแสดงกิจกรรมล่าสุด โปรแกรมที่ไม่ปรากฏใน BAM
  อาจเคยทำงานเมื่อนานมาแล้วก็ได้
- **พาธอุปกรณ์ ไม่ใช่อักษรไดรฟ์** `\Device\HarddiskVolume3` คือวอลุ่มที่สามที่ Windows
  กำหนดหมายเลข ไม่ใช่ `C:` โดย `DevicePath` และ `Path` แยกอุปกรณ์ออกจากพาธบนอุปกรณ์นั้น ให้จับคู่
  วอลุ่มกับ `MountedDevices`, `$MFT` หรืออาร์ติแฟกต์อื่นก่อนระบุอักษรไดรฟ์
- **ไม่ใช่ทุกโปรแกรม** BAM และ DAM ติดตามโปรแกรมในเซสชันของผู้ใช้ บริการส่วนใหญ่จึงไม่
  ปรากฏ รายการ `S-1-5-18` โดยทั่วไปเป็นโพรเซสของการติดตั้งและหน้าจอลงชื่อเข้าใช้
  (`FirstLogonAnim.exe`) และรายการ `S-1-5-90-0-<n>` เป็นของ Desktop Window Manager (`DWM-<n>`)
- **Control set** โดยค่าเริ่มต้นจะอ่าน control set ที่ Windows ใช้ล่าสุด `--all-control-sets`
  จะอ่านทุกชุด control set ที่เก่ากว่าอาจเก็บรายการที่ชุดปัจจุบันไม่มีแล้ว
- **ชื่อผู้ใช้** ชื่อของบัญชีภายในเครื่องมาจาก `SAM` (SID ของคอมพิวเตอร์และ RID ของ
  บัญชี) ส่วนชื่อของบัญชีโดเมนมาจากโฟลเดอร์โปรไฟล์ใน `ProfileList` โดย `UserSource`
  ระบุแหล่งที่มา (`sam`, `profile_list`, `well_known`) หากไม่มีข้อมูลเหล่านี้จะแสดงเพียง SID เท่านั้น RID
  ที่ไม่มีอยู่ใน `SAM` แล้ว (บัญชีที่ถูกลบ) จะไม่มีชื่อ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bam <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM hives
  -f, --file <FILE>      Parse one SYSTEM hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Filtering:
      --all-control-sets  Every control set, not only the one in use

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bam -d evidence/ -o bam.csv
juntei parse-registry bam -f SYSTEM -t jsonl \
  | jq 'select(.Path? // "" | test("\\\\(Users|ProgramData|Temp)\\\\"; "i"))'
```

สรุปผลจะแสดงผู้ใช้แต่ละคนพร้อม SID ชื่อ และจำนวนรายการ รวมถึงโปรแกรมที่ทำงานล่าสุด
(เรียงจากใหม่ไปเก่า)

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | โฮสต์และไฮฟ์ `SYSTEM` |
| `ControlSet`, `IsCurrentControlSet` | control set ที่มีรายการนั้น และเป็นชุดที่ Windows ใช้ล่าสุดหรือไม่ |
| `Moderator` | `bam` หรือ `dam` |
| `UserSID`, `User`, `UserSource` | SID ที่ใช้ตั้งชื่อคีย์ ชื่อผู้ใช้ และแหล่งที่มาของชื่อ (`sam`, `profile_list`, `well_known`) |
| `Program` | ชื่อของค่าตามที่ Windows เขียน ได้แก่ พาธอุปกรณ์หรือ ID ของแอปแบบแพ็กเกจ |
| `DevicePath`, `Path` | สำหรับพาธอุปกรณ์ คืออุปกรณ์ (`\Device\HarddiskVolume3`) และพาธบนอุปกรณ์ (`\Windows\System32\cmd.exe`) ว่างสำหรับแอปแบบแพ็กเกจ |
| `LastRunUTC` | เวลาที่โปรแกรมทำงานครั้งล่าสุด (ละเอียดถึง 100 ns) |
| `KeyPath`, `KeyLastWrittenUTC` | คีย์ของผู้ใช้และเวลาเขียนล่าสุด คือการอัปเดตครั้งล่าสุดของรายการใดรายการหนึ่งในคีย์ |
| `DataRaw` | ไบต์ของค่า (hex) เมื่อไม่มีเวลาอยู่ ได้แก่ ไม่ใช่ `REG_BINARY` หรือมีน้อยกว่า 8 ไบต์ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ข้อมูลที่ไม่มีเวลา และค่าที่อ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. เรียงตาม `LastRunUTC` แล้วดูโปรแกรมในตำแหน่งที่ผู้ใช้เขียนได้ (`\Users\`, `Downloads`,
   `Temp`, `ProgramData`) บนอุปกรณ์อื่นที่ไม่ใช่วอลุ่มระบบ และเครื่องมือดูแลระบบและ
   เครื่องมือเข้าถึงระยะไกล
2. ตรวจสอบว่าผู้ใช้คนใดเรียกใช้โปรแกรมเหล่านั้น และบัญชีที่ไม่ตรงกับโปรไฟล์ปัจจุบัน
3. เชื่อมโยงกับ Prefetch (จำนวนครั้งที่ทำงานและการทำงานครั้งก่อน), UserAssist (การเริ่มจาก Explorer), Amcache
   และ ShimCache (การมีอยู่), `$MFT` (ไฟล์ของโปรแกรม) และ event log
4. ใช้ `--all-control-sets` และ `parse-registry read --recover-deleted` สำหรับค่า BAM ที่ถูกลบ

## รายละเอียดเพิ่มเติม { #further-details }

- [libyal winreg-kb: Background activity moderator (BAM)](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Background-activity-moderator.md)
- [Microsoft: Security identifiers (SID ที่รู้จักกันทั่วไป)](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)

FILETIME ของการทำงานล่าสุดที่อยู่นอกช่วงจะทำให้ `LastRunUTC` ว่าง และเก็บรักษาค่าเดิมไว้ใน `DataRaw` พร้อมคำอธิบายใน `Problems` สรุปกิจกรรมล่าสุดจะระบุแต่ละโฮสต์และแยกข้อมูลที่สังเกตได้จากแต่ละไฮฟ์/ผู้ใช้ออกจากกัน
