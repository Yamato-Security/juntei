# ความลับ LSA { #lsa-secrets }

## สิ่งเหล่านี้คืออะไร { #what-they-are }

Local Security Authority (LSA) เก็บความลับ (secret) ไว้ในไฮฟ์ `SECURITY` ซึ่งรวมถึง
รหัสผ่านสำหรับ autologon รหัสผ่านของบัญชีที่บริการใช้ทำงาน รหัสผ่านโดเมนของบัญชีคอมพิวเตอร์
คีย์ระบบของ DPAPI และคีย์ที่ใช้เข้ารหัสข้อมูลการเข้าสู่ระบบโดเมนที่แคชไว้
นอกจากนี้ยังเก็บข้อมูลการเข้าสู่ระบบโดเมนที่แคชไว้ เพื่อให้ผู้ใช้โดเมนเข้าสู่ระบบได้โดยไม่ต้องติดต่อ domain controller

`parse-registry lsa-secrets` แสดงรายการความลับที่ระบบเก็บไว้และเวลาที่ตั้งค่า รวมถึง
จำนวนช่องการเข้าสู่ระบบที่แคชไว้ที่ระบบเก็บไว้และใช้งานอยู่ **คำสั่งนี้อ่านเฉพาะชื่อและเวลา: ไม่มีการถอดรหัสใด ๆ และ
ไม่มีการอ่านหรือเขียนไบต์ของความลับใด ๆ** Juntei อ่านหลักฐาน แต่ไม่ได้ดึงข้อมูลรับรอง (credential) ออกมา

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ระบบเก็บความลับชื่อนี้ไว้ ซึ่งถูกตั้งค่า
ณ เวลานี้ ความลับของบริการ (`_SC_<service>`) หมายความว่า LSA เก็บรหัสผ่านของบัญชี
ที่บริการนั้นใช้ทำงาน

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SECURITY
Policy\Secrets\<name>\CurrVal    the current value (encrypted)
Policy\Secrets\<name>\OldVal     the previous value
Policy\Secrets\<name>\CupdTime   when the current value was set (FILETIME)
Policy\Secrets\<name>\OupdTime   when the previous value was set
Cache\NL$1 … NL$n                cached domain logons, one slot each
Cache\NL$Control                 how many slots Windows keeps
```

| ชื่อ | ประเภท | คืออะไร |
|---|---|---|
| `DefaultPassword` | `autologon` | รหัสผ่านที่ Winlogon ใช้เข้าสู่ระบบโดยอัตโนมัติ |
| `_SC_<service>` | `service_account` | รหัสผ่านของบัญชีที่ `<service>` ใช้ทำงาน |
| `$MACHINE.ACC` | `machine_account` | รหัสผ่านของบัญชีคอมพิวเตอร์ในโดเมน |
| `DPAPI_SYSTEM` | `dpapi_system` | คีย์ DPAPI ของระบบ |
| `NL$KM` | `cached_logon_key` | คีย์ที่ใช้เข้ารหัสข้อมูลการเข้าสู่ระบบที่แคชไว้ |
| `L$_SQSA_<SID>` | `security_questions` | คำถามความปลอดภัยของบัญชีภายในเครื่อง (Windows 10 ขึ้นไป) |
| `RasDialParams!<SID>#n`, `L$_Ras…` | `ras_credentials` | ข้อมูลรับรองสำหรับ dial-up และ VPN |
| `L$…`, `G$…`, `M$…` | `local`, `global`, `machine` | ข้อมูลส่วนตัวอื่น ๆ แยกตาม prefix ที่ `LsaStorePrivateData` กำหนด |

ให้เก็บรวบรวม `SECURITY` พร้อม transaction log และไฮฟ์ `SYSTEM` ของระบบ ซึ่งระบุชื่อ
โฮสต์

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **ชื่อและเวลา ไม่ใช่ค่า** การที่ค่ามีข้อมูลอยู่หรือไม่นั้นพิจารณาจากขนาดของค่า เนื้อหาของค่า
  ไม่ได้ถูกอ่าน
- **`DefaultPassword` ไม่ได้หมายถึง autologon** ความลับนี้มีอยู่บนระบบส่วนใหญ่ รวมถึง
  ระบบที่ไม่เคยเข้าสู่ระบบโดยอัตโนมัติ
  `Microsoft\Windows NT\CurrentVersion\Winlogon` ของ `SOFTWARE` (`AutoAdminLogon`, `DefaultUserName`) ระบุ
  ว่ามีการกำหนดค่า autologon หรือไม่ `CurrentSetUTC` ที่เพิ่งเกิดขึ้นเป็นการเปลี่ยนแปลงที่ควรหาคำอธิบาย
- **ความลับของบริการ:** `_SC_<service>` จะปรากฏเมื่อบริการถูกตั้งค่าให้ทำงานในฐานะบัญชีที่มี
  รหัสผ่าน ให้เปรียบเทียบกับ `parse-registry services` (`ObjectName`) และมองหาบริการ
  ที่เพิ่งติดตั้งหรือไม่รู้จัก ความลับอาจยังคงอยู่หลังจากบริการถูกลบไปแล้ว
- **เวลา:** `CurrentSetUTC` และ `OldSetUTC` คือเวลาที่ Windows ตั้งค่าปัจจุบันและค่าก่อนหน้า
  รหัสผ่าน `$MACHINE.ACC` เปลี่ยนทุก 30 วันตามค่าเริ่มต้น ดังนั้นเวลาของมันจะเปลี่ยนไปเรื่อย ๆ
  `KeyLastWrittenUTC` และเวลาของคีย์ `CurrVal`/`OldVal` เกี่ยวกับเวลาที่ความลับ
  เปลี่ยนแปลงครั้งล่าสุด
- **การเข้าสู่ระบบที่แคชไว้:** ช่องจะถือว่าใช้งานอยู่เมื่อระเบียนในช่องระบุชื่อผู้ใช้ ส่วนหัวของระเบียนไม่ได้
  ถูกเข้ารหัส และจะอ่านเฉพาะความยาวของชื่อผู้ใช้เท่านั้น `CachedLogonsKept` คือจำนวนของ `NL$Control`
  ซึ่งเป็นไปตาม `CachedLogonsCount` (ค่าเริ่มต้นคือ 10) ช่องที่ใช้งานอยู่หมายความว่าผู้ใช้โดเมนได้เข้าสู่ระบบ
  แบบ interactive และข้อมูลรับรองของพวกเขาสามารถถูกถอดรหัส (crack) แบบออฟไลน์จากไฮฟ์ได้ ชื่อของผู้ใช้
  ถูกเข้ารหัสและไม่ได้ถูกถอดรหัส เฉพาะระเบียนไบนารีที่ผ่านการตรวจสอบซึ่งมีส่วนหัวครบถ้วนและความยาวชื่อผู้ใช้สอดคล้องกันเท่านั้นที่นับว่าใช้งานอยู่ หาก `Problems` รายงานช่องที่เสียหาย จำนวนที่ใช้งานอยู่จะครอบคลุมเฉพาะช่องที่ผ่านการตรวจสอบเท่านั้น อย่าตีความว่าช่องที่เสียหายเป็นช่องว่าง

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry lsa-secrets <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SECURITY hives
  -f, --file <FILE>      Parse one SECURITY hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry lsa-secrets -d collection/ -o lsa-secrets.csv
juntei parse-registry lsa-secrets -f SECURITY -t jsonl | jq 'select(.Kind == "service_account")'
```

สำหรับแต่ละไฮฟ์ ส่วนสรุปจะแสดงความลับแต่ละรายการพร้อมประเภทและเวลาที่ตั้งค่า จากนั้นระบุชื่อ
บริการที่ LSA เก็บรหัสผ่านบัญชีไว้ และนับจำนวนช่องการเข้าสู่ระบบที่แคชไว้ที่ใช้งานอยู่

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath` | โฮสต์ (จากไฮฟ์ `SYSTEM` ของระบบ) และไฮฟ์ `SECURITY` |
| `Secret`, `Kind` | ชื่อของความลับและประเภท (ดูตารางด้านบน) โดยระเบียนของการเข้าสู่ระบบที่แคชไว้จะเป็น `Cache` และ `cached_logons` |
| `ServiceName`, `UserSID` | บริการที่ความลับ `_SC_` เป็นของ และ SID ของผู้ใช้ที่ได้จากชื่อ (`L$_SQSA_`, `RasDialParams`) |
| `CurrentSetUTC`, `OldSetUTC` | `CupdTime` และ `OupdTime`: เวลาที่ตั้งค่าปัจจุบันและค่าก่อนหน้า |
| `HasCurrentValue`, `HasOldValue` | `CurrVal` และ `OldVal` มีข้อมูลหรือไม่ |
| `CurrentValueLastWrittenUTC`, `OldValueLastWrittenUTC`, `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ `CurrVal`, `OldVal` และคีย์ของความลับ |
| `CachedLogons`, `CachedLogonsInUse`, `CachedLogonsKept` | ระเบียนของการเข้าสู่ระบบที่แคชไว้: จำนวนช่อง ช่องที่ใช้งานอยู่ และจำนวนของ `NL$Control` |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | เวลาที่ผิดรูปแบบหรืออยู่นอกช่วง (แสดงเฉพาะเมทาดาทาของเวลา) ส่วนหัวของแคชที่เสียหาย ชนิดของค่าแคชที่ไม่ถูกต้อง และคีย์ที่อ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. มองหาความลับ `_SC_` ของบริการที่ไม่รู้จักหรือเพิ่งติดตั้ง และจับคู่กับ
   `parse-registry services` และเหตุการณ์ 7045 (มีการติดตั้งบริการ)
2. ตรวจสอบเวลาของ `DefaultPassword` เทียบกับการตั้งค่า autologon ของ `Winlogon`
3. บนเครื่องที่เป็นสมาชิกโดเมน ให้บันทึกช่องการเข้าสู่ระบบที่แคชไว้ที่ใช้งานอยู่ และเวลาที่ `Cache` ถูกเขียนครั้งล่าสุด
4. เปรียบเทียบเวลาของความลับกับไทม์ไลน์: ความลับที่ถูกตั้งค่าในช่วงเวลาของการบุกรุกอาจ
   บ่งชี้บัญชีบริการใหม่ การเปลี่ยนแปลง autologon หรือบัญชีเครื่อง

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Private data object](https://learn.microsoft.com/en-us/windows/win32/secmgmt/private-data-object)
- [Microsoft: ข้อมูลการเข้าสู่ระบบโดเมนที่แคชไว้](https://learn.microsoft.com/en-us/troubleshoot/windows-server/user-profiles-and-logon/cached-domain-logon-information)
- [libyal winreg-kb: Cached credentials](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Cached-credentials.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
