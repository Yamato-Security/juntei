# ความเข้ากันได้ของแอปพลิเคชัน { #application-compatibility }

## สิ่งที่ Windows เก็บไว้ { #what-windows-keeps }

นอกเหนือจาก ShimCache (`parse-registry shimcache`) และ Amcache (`parse-registry amcache`)
ฟีเจอร์ความเข้ากันได้ของแอปพลิเคชันของ Windows ยังเก็บข้อมูลต่อไปนี้ไว้ใต้
`Microsoft\Windows NT\CurrentVersion\AppCompatFlags`:

| แหล่งข้อมูล | ไฮฟ์และคีย์ | สิ่งที่แสดง |
|---|---|---|
| `layers` | `SOFTWARE` (และ `Wow6432Node`) และ `NTUSER.DAT` `Layers` | โปรแกรมที่ตั้งค่าให้ทำงานด้วย compatibility layer: `RUNASADMIN` (ทำงานในฐานะผู้ดูแลระบบเสมอ), `WIN7RTM`, `HIGHDPIAWARE`, … |
| `pca_store` | `NTUSER.DAT` `Compatibility Assistant\Store` (Windows 8 ขึ้นไป) | โปรแกรมที่ Program Compatibility Assistant เห็นว่าผู้ใช้เรียกใช้ |
| `pca_persisted` | `NTUSER.DAT` `Compatibility Assistant\Persisted` (Windows 7) | ข้อมูลเดียวกันบน Windows 7 |
| `cit_module` | `SOFTWARE` `CIT\Module\<module>` | โปรแกรมที่โหลดโมดูลที่ถูกติดตาม และเวลาที่โหลดครั้งล่าสุด |
| `cit_system` | `SOFTWARE` `CIT\System` (Windows 7 ถึง Windows 10 รุ่นแรก ๆ) | พาธโปรแกรมและบรรทัดคำสั่งในฐานข้อมูล CIT ที่ถูกบีบอัด |
| `installed_sdb` | `SOFTWARE` `InstalledSDB\{GUID}` | ฐานข้อมูล shim แบบกำหนดเองที่ติดตั้งด้วย `sdbinst`: ไฟล์ คำอธิบาย และเวลาติดตั้ง |
| `custom_sdb` | `SOFTWARE` `Custom\<program>` | ฐานข้อมูล shim แบบกำหนดเองที่ใช้กับแต่ละโปรแกรม พร้อมเวลาติดตั้ง |

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> layer และฐานข้อมูล shim เป็น
การกำหนดค่าที่เปลี่ยนวิธีการทำงานของโปรแกรม ส่วนรายการของ Program Compatibility Assistant และ CIT
แสดงว่าโปรแกรมได้ทำงาน (หรือโหลดโมดูล) ในฐานะผู้ใช้รายนี้หรือบนระบบนี้

ฐานข้อมูล shim แบบกำหนดเองเป็นเทคนิคการคงอยู่ (persistence) และการแทรกโค้ด (MITRE ATT&CK T1546.011): shim
สามารถโหลด DLL เข้าไปในโปรแกรม หรือเริ่มโปรแกรมอื่นทุกครั้งที่โปรแกรมนั้นทำงาน

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers                <program> = <layers>
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\Module\<module>   <program> = FILETIME
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\CIT\System            <id> = compressed database
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\InstalledSDB\{GUID}   DatabasePath, …
  Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Custom\<program.exe>  {GUID}.sdb = FILETIME
C:\Users\<user>\NTUSER.DAT
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Store
  Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Compatibility Assistant\Persisted
```

ให้เก็บรวบรวม `SOFTWARE`, `NTUSER.DAT` ทุกไฟล์พร้อม transaction log และไฟล์ `.sdb` ที่พาธของฐานข้อมูล
shim ระบุไว้ (โดยปกติคือ `C:\Windows\AppPatch\Custom\`)

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **การขยายข้อมูล CIT มีขีดจำกัด** ค่าที่ระบุขนาดหลังขยายเกิน 64 MiB จะถูกเก็บไว้
  เป็นไบต์ดิบพร้อมข้อความวินิจฉัย การบีบอัดที่ผิดรูปแบบและขนาดที่ไม่ตรงกันจะถูกรายงาน


- **รายการของ Program Compatibility Assistant ไม่มีเวลาที่เรียกใช้** ข้อมูล `Store` (`SACP` ซึ่งเก็บไว้ใน
  `DataRaw`) มีเวลาหนึ่งค่า แต่บน Windows 8.1, 10 และ 11 ค่านั้นเหมือนกันสำหรับทุกโปรแกรมในไฮฟ์เดียวกัน
  และสำหรับระบบอื่นก็เช่นกัน: ค่านี้ไม่ใช่เวลาที่โปรแกรมทำงาน มีเพียงเวลาเขียนล่าสุดของคีย์ `Store`
  เท่านั้นที่ระบุวันเวลาของการเปลี่ยนแปลงครั้งล่าสุด
- **โปรแกรมบนสื่อแบบถอดได้** จะมีชื่อเป็น `SIGN.MEDIA=<id> <path>`: `Path` คือพาธบน
  สื่อนั้น และ `Details` มี id ของสื่อ
- **ฐานข้อมูล CIT** ถูกบีบอัดด้วย LZNT1 และโครงสร้างไม่มีเอกสารเผยแพร่สาธารณะ: Juntei
  แสดงรายการพาธโปรแกรมและบรรทัดคำสั่งที่พบในแต่ละฐานข้อมูล ครั้งเดียวต่อฐานข้อมูล โดยไม่มีจำนวนครั้งหรือ
  เวลา บรรทัดคำสั่งเป็นระเบียนแยกของตัวเอง ไม่ได้ผูกกับโปรแกรมของมัน Windows 10 ขึ้นไปเก็บ
  ข้อมูลส่วนนี้ไว้น้อยมาก
- **ฐานข้อมูล shim** ให้ตรวจสอบทุกระเบียน `custom_sdb` และ `installed_sdb`: ซอฟต์แวร์ที่ถูกต้อง
  แทบไม่เคยติดตั้งฐานข้อมูลนี้ ตัวไฟล์ `.sdb` เองจะบอกว่า shim ทำอะไร
- **Layer** ที่ผู้ใช้ตั้งค่าจะมีผลเฉพาะกับผู้ใช้รายนั้น ส่วนของ `SOFTWARE` มีผลกับทุกคน `RUNASADMIN` ทำให้
  โปรแกรมขอยกระดับสิทธิ์ทุกครั้ง

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry app-compat <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SOFTWARE and NTUSER.DAT hives
  -f, --file <FILE>      Parse one SOFTWARE or NTUSER.DAT hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry app-compat -d evidence/ -o appcompat.csv
juntei parse-registry app-compat -d evidence/ -t jsonl \
  | jq 'select(.Source == "installed_sdb" or .Source == "custom_sdb")'
```

ส่วนสรุปจะแสดงรายการฐานข้อมูล shim แบบกำหนดเอง โปรแกรมที่มี layer และการใช้งานครั้งล่าสุด
ของโมดูล CIT

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | โฮสต์ ไฟล์ไฮฟ์ และ `SOFTWARE` หรือ `NTUSER.DAT` |
| `User`, `UserSID` | ชื่อโฟลเดอร์โปรไฟล์ของไฮฟ์ผู้ใช้ และ SID จาก `ProfileList` ของระบบ |
| `Source`, `Scope` | `layers`, `pca_store`, `pca_persisted`, `cit_module`, `cit_system`, `installed_sdb`, `custom_sdb`; `machine` หรือ `user` |
| `Path` | โปรแกรม (ชื่อของค่า) พาธหรือบรรทัดคำสั่งในฐานข้อมูล CIT หรือโปรแกรมที่ฐานข้อมูล shim มีผล |
| `Layers` | compatibility layer ตามที่เขียนไว้ (`~ RUNASADMIN`) |
| `LastSeenUTC` | เวลาที่โปรแกรมโหลดโมดูล CIT ครั้งล่าสุด |
| `SdbGUID`, `SdbPath`, `SdbInstalledUTC` | GUID ไฟล์ และเวลาติดตั้งของฐานข้อมูล shim |
| `Details` | ชื่อโมดูล CIT ชื่อค่าของฐานข้อมูล CIT id สื่อแบบถอดได้ของโปรแกรม (`SignMedia`) หรือ `DatabaseDescription` และ `DatabaseType` ของฐานข้อมูล shim |
| `KeyPath`, `KeyLastWrittenUTC`, `ValueName`, `WOW64` | ที่มาของระเบียน เวลาที่คีย์นั้นเปลี่ยนแปลงครั้งล่าสุด และอยู่ใต้ `Wow6432Node` หรือไม่ |
| `DataRaw` | ข้อมูล `SACP` ของค่า `Store` หรือข้อมูลที่ไม่เป็นไปตามที่ควรจะเป็น (hex) |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | ข้อมูลที่ถอดรหัสหรืออ่านไม่ได้ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ตรวจทานฐานข้อมูล shim แบบกำหนดเองทุกรายการ: ไฟล์ `.sdb` เวลาติดตั้ง และโปรแกรมที่มีผล
   แล้วเชื่อมโยงกับ `sdbinst.exe` ใน Prefetch, Amcache และเหตุการณ์การสร้างโพรเซส
2. มองหาโปรแกรมในโฟลเดอร์ที่ผู้ใช้เขียนได้ในรายการของ Program Compatibility Assistant และ
   โปรแกรมจากสื่อแบบถอดได้
3. ตรวจสอบ `RUNASADMIN` และ layer อื่น ๆ บนโปรแกรมที่ผิดปกติ
4. เชื่อมโยงกับ ShimCache, Amcache, BAM, Prefetch และ UserAssist เพื่อหาเวลาการเรียกใช้งาน

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: ฐานข้อมูลความเข้ากันได้ของแอปพลิเคชัน](https://learn.microsoft.com/en-us/windows/win32/devnotes/application-compatibility-database)
- [MITRE ATT&CK T1546.011: Application Shimming](https://attack.mitre.org/techniques/T1546/011/)
- [Microsoft: MS-XCA (LZNT1)](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-xca/)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
