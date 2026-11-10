# ข้อมูลระบบ { #system-information }

## ข้อมูลนี้คืออะไร { #what-it-is }

ไฮฟ์ระดับเครื่องทั้งสี่ของระบบบอกสิ่งที่นักวิเคราะห์จำเป็นต้องรู้ก่อนอ่านไทม์ไลน์ของระบบนั้น ได้แก่

- ชื่อของระบบ (NetBIOS, DNS host name, DNS domain) และโดเมนหรือเวิร์กกรุ๊ปที่ระบบอยู่
- เวอร์ชัน build และ update revision ของ Windows และเวลาที่ติดตั้ง Windows หรืออัปเกรดครั้งล่าสุด
- เขตเวลาของระบบ ซึ่งบอกเวลาท้องถิ่นที่ระบบใช้บันทึก
- เวลาที่ระบบปิดเครื่องอย่างสมบูรณ์ครั้งล่าสุด
- สถาปัตยกรรมโปรเซสเซอร์และ code page
- ผู้ใช้ที่ลงชื่อเข้าใช้ครั้งล่าสุด และโปรไฟล์ของผู้ใช้ทุกคนที่เคยลงชื่อเข้าใช้
- SID ของคอมพิวเตอร์ (บัญชีภายในเครื่องคือ SID นี้บวก RID) และ SID ของโดเมน
- เวอร์ชัน Windows ที่ระบบถูกอัปเกรดมา

`parse-registry system-info` เขียนหนึ่งเรคคอร์ดต่อหนึ่งระบบซึ่งมีข้อมูลทั้งหมดนี้

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> การกำหนดค่า ณ เวลาที่เก็บรวบรวมไฮฟ์
(หรือที่เขียนครั้งล่าสุด) เวลาของคีย์บอกว่าส่วนต่าง ๆ เปลี่ยนแปลงครั้งล่าสุดเมื่อใด

## ตำแหน่ง { #location }

```text
C:\Windows\System32\config\SYSTEM     (the control set Windows used last: Select\Current)
  ControlSet00N\Control\ComputerName\ComputerName     ComputerName
  ControlSet00N\Services\Tcpip\Parameters             Hostname, Domain
  ControlSet00N\Control\TimeZoneInformation           TimeZoneKeyName, Bias, ActiveTimeBias, …
  ControlSet00N\Control\Windows                       ShutdownTime
  ControlSet00N\Control\Session Manager\Environment   PROCESSOR_ARCHITECTURE
  ControlSet00N\Control\Nls\CodePage                  ACP, OEMCP
  Setup\Source OS (Updated on M/D/YYYY HH:MM:SS)      the version before each upgrade

C:\Windows\System32\config\SOFTWARE
  Microsoft\Windows NT\CurrentVersion                 ProductName, EditionID, DisplayVersion,
                                                      CurrentBuild, UBR, InstallDate, InstallTime, …
  Microsoft\Windows NT\CurrentVersion\ProfileList     one key per profile (SID)
  Microsoft\Windows\CurrentVersion\Authentication\LogonUI   LastLoggedOnUser, …

C:\Windows\System32\config\SAM
  SAM\Domains\Account                                 V: the computer's SID at its end

C:\Windows\System32\config\SECURITY
  Policy\PolAcDmN, PolAcDmS                           the computer's name and SID
  Policy\PolPrDmN, PolPrDmS                           its domain (or workgroup) and the domain's SID
  Policy\PolDnDDN, PolDnTrN, PolDnDmG                 the domain's and forest's DNS names, GUID
  Policy\PolMachineAccountS                           the computer account's SID in the domain
```

ให้เก็บรวบรวมไฮฟ์ทั้งสี่พร้อมบันทึกธุรกรรม ไฮฟ์ใดไฮฟ์หนึ่งเพียงลำพังก็ให้ข้อมูลที่ไฮฟ์นั้นเก็บไว้ได้

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **เวลาในรีจิสทรีเป็น UTC ส่วนเขตเวลาใช้กับข้อมูลอื่นทั้งหมด** เวลาของคีย์ FILETIME และเวลาของ
  เหตุการณ์ส่วนใหญ่เป็น UTC บันทึกแบบข้อความ บันทึกของแอปพลิเคชันบางตัว และวันที่ที่เขียนอยู่ในชื่อหรือสตริง
  (เช่น `Source OS (Updated on …)`) อาจเป็นเวลาท้องถิ่น `Bias` คือจำนวนนาทีจากเวลามาตรฐานท้องถิ่น
  ไปยัง UTC (`UTC = local + Bias`) `UtcOffset` และ `DaylightUtcOffset` ให้ค่า offset ทั้งสองค่า
  และ `TimeZoneDescription` บอกว่าเวลาออมแสงเริ่มต้นและสิ้นสุดเมื่อใด
- **เขตเวลาเปลี่ยนแปลงได้** `TimeZoneLastWrittenUTC` คือเวลาเขียนล่าสุดของคีย์ Windows ยัง
  เขียนคีย์นี้ใหม่เองเมื่ออัปเดต `ActiveTimeBias` (offset ที่มีผลในขณะนั้น) ดังนั้น
  เวลาที่เพิ่งเปลี่ยนจึงไม่ได้พิสูจน์ว่ามีผู้เปลี่ยนเขตเวลา ให้เปรียบเทียบกับ event log: Security 4616
  (เวลาของระบบถูกเปลี่ยน) และเหตุการณ์ `Microsoft-Windows-Kernel-General` ใน System log
- **`StandardStart` และ `DaylightStart` เป็น `TIME_FIELDS` ของเคอร์เนล ไม่ใช่ `SYSTEMTIME`**
  วันในสัปดาห์เป็นฟิลด์สุดท้าย ไม่ใช่ฟิลด์ที่สาม (เหมือนในค่า `TZI` ของเขตเวลาใน `SOFTWARE`) ใน
  เขตเวลาของสหรัฐฯ ทั้งสองรูปแบบอ่านได้ค่าเดียวกัน เพราะวันในสัปดาห์เป็นวันอาทิตย์ (0) Juntei ถอดรหัสค่าเหล่านี้
  เป็น `TIME_FIELDS`: ระบบสาธารณะของเกาะอีสเตอร์ (วันเสาร์แรก เวลา 22:00) แสดงให้เห็น
  ความแตกต่างนี้ เดือนที่เป็น 0 หมายถึงไม่มีเวลาออมแสง
- **Windows 11 แสดงเป็น "Windows 10"** `ProductName` ยังคงเป็น `Windows 10 …` บน Windows 11 build 22000
  ขึ้นไปคือ Windows 11 และข้อมูลสรุปจะระบุเช่นนั้น `CurrentVersion` เป็น `6.3` บน Windows 8.1, 10
  และ 11 ส่วน `Version` จะใช้ `CurrentMajorVersionNumber` (10) เมื่อมีค่านี้อยู่
- **`InstallDate` เปลี่ยนไปตามการอัปเกรดฟีเจอร์** การอัปเกรดแบบ in-place ไปยังรุ่นใหม่ (เช่น 1809 เป็น
  21H2) จะรีเซ็ตค่านี้ รุ่นที่การอัปเกรดแต่ละครั้งแทนที่ พร้อม `InstallDate` ของรุ่นนั้นเอง จะอยู่
  ใน `PreviousVersions` (`Setup\Source OS`) `InstallDate` ที่เก่าที่สุดในกลุ่มนี้ใกล้เคียงกับ
  การติดตั้งครั้งแรกมากที่สุด วันที่ในชื่อคีย์ `Source OS` ไม่ได้ระบุเขตเวลา
- **`ShutdownTime` คือการปิดเครื่องอย่างสมบูรณ์ครั้งล่าสุด** การแครชหรือไฟดับไม่ได้อัปเดตค่านี้
  ให้เปรียบเทียบกับ event log (6006, 6008, 1074) และกับเวลาเขียนล่าสุดของไฮฟ์
- **`LastLoggedOnUser` คือผู้ใช้ล่าสุดของหน้าจอลงชื่อเข้าใช้** ไม่ใช่การเข้าสู่ระบบทุกครั้ง Remote Desktop,
  `runas` บริการ และงานตามกำหนดเวลาไม่ได้ตั้งค่านี้ทั้งหมด `LogonUiLastWrittenUTC` คือเวลาโดยประมาณที่
  ค่านี้ถูกตั้งครั้งล่าสุด
- **โปรไฟล์:** `ProfileList` แสดงผู้ใช้ (และบริการ) ทุกรายที่เคยมีโปรไฟล์บน
  ระบบ รวมถึงบัญชีที่ถูกลบจนกว่าโปรไฟล์จะถูกลบออก Windows 10 และรุ่นใหม่กว่าเขียน
  `LocalProfileLoadTime` และ `LocalProfileUnloadTime` ซึ่งเป็นเวลาโดยประมาณของการลงชื่อเข้าใช้และออกจากระบบครั้งล่าสุด
- **SID:** บัญชีภายในเครื่องคือ `MachineSid-<RID>` บัญชีโดเมนคือ `DomainSid-<RID>`
  `MachineSid` มาจาก `SAM` (หรือจาก `PolAcDmS` ของ `SECURITY` ในกรณีอื่น) ค่าทั้งสองควรตรงกัน
  เมื่อไม่ตรงกัน `Problems` จะแจ้งไว้ เช่นเดียวกับกรณีไฮฟ์มาจากต่างระบบ `DomainSid` จะว่าง
  ในเวิร์กกรุ๊ป
- **หนึ่งไดเรกทอรี หนึ่งระบบ** เมื่อใช้ `-d` ไฮฟ์ในไดเรกทอรีเดียวกันถือเป็นของระบบเดียว
  สำเนา `RegBack` และไฮฟ์ชนิดเดียวกันชุดที่สอง (`SYSTEM.old`) จะเป็นเรคคอร์ดแยกของตนเอง
  เมื่อใช้ `-f` ไฮฟ์ที่ระบุถือเป็นของระบบเดียว และจะค้นหาไฮฟ์อื่นที่อยู่ข้างกัน
- **Control set:** จะอ่านเฉพาะ control set ที่ Windows ใช้ล่าสุดเท่านั้น `parse-registry read` จะแสดง
  control set อื่น ๆ

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry system-info <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find SYSTEM, SOFTWARE, SAM and SECURITY hives
  -f, --file <FILE>      A hive of one system; repeat for each
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry system-info -d collection/ -o system-info.csv
juntei parse-registry system-info -f SYSTEM -f SOFTWARE -t jsonl \
  | jq '{Host, TimeZoneDescription, ProductName, Build, LastShutdown}'
```

ข้อมูลสรุปจะแสดงสำหรับแต่ละระบบ ได้แก่ เวอร์ชัน Windows เวลาที่ติดตั้ง เขตเวลาในรูป
คำบรรยาย offset ที่มีผลเมื่อคีย์ถูกเขียนครั้งล่าสุด การปิดเครื่องครั้งล่าสุด โดเมนหรือ
เวิร์กกรุ๊ป ผู้ใช้ล่าสุดที่ลงชื่อเข้าใช้ SID จำนวนโปรไฟล์ และเวอร์ชันที่ระบบ
ถูกอัปเกรดมา `auto-parse` ยังพิมพ์เขตเวลาของแต่ละระบบเมื่อแสดงรายการระบบที่
ค้นพบด้วย

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `SoftwarePath`, `SamPath`, `SecurityPath` | โฮสต์ และไฮฟ์ที่อ่าน (`SourcePath` คือไฮฟ์ `SYSTEM` หรือไฮฟ์แรกที่อ่านในกรณีอื่น) |
| `CompName`, `Hostname`, `Domain` | ชื่อ NetBIOS (`SYSTEM` หรือ `PolAcDmN` ของ `SECURITY` ในกรณีอื่น) DNS host name และ DNS domain (`Tcpip\Parameters`) |
| `PrimaryDomain`, `DNS-Domain` | โดเมนที่คอมพิวเตอร์เข้าร่วม หรือเวิร์กกรุ๊ป (`PolPrDmN`) และชื่อ DNS ของโดเมน (`PolDnDDN`) |
| `ProductName`, `Edition`, `DisplayVersion`, `Version`, `Build`, `Ubr`, `ServicePack`, `InstallationType` | `CurrentVersion`: ชื่อผลิตภัณฑ์ตามที่เขียนไว้ `EditionID`, `DisplayVersion` (หรือ `ReleaseId`) เวอร์ชันหลักและรอง (หรือ `CurrentVersion`) build, update build revision, `CSDVersion` และ `Client`, `Server` หรือ `Server Core` |
| `InstallDateUTC`, `InstallTimeUTC` | `InstallDate` (ละเอียดถึงวินาที) และ `InstallTime` (ละเอียดถึง 100 ns บน Windows 10 และรุ่นใหม่กว่า): การติดตั้งหรือการอัปเกรดฟีเจอร์ครั้งล่าสุด |
| `RegisteredOwner`, `RegisteredOrganization`, `SystemRoot` | ตามที่เขียนไว้ |
| `TimeZone`, `TimeZoneDescription` | `TimeZoneKeyName` (หรือ `StandardName`) และเขตเวลาในรูปคำบรรยายพร้อม offset และวันที่ของเวลาออมแสง |
| `UTC-Offset`, `DaylightUTC-Offset` | offset ของเวลามาตรฐานท้องถิ่นจาก UTC และของเวลาออมแสง (ว่างหากไม่มีเวลาออมแสง) |
| `BiasMinutes`, `ActiveBiasMinutes`, `StandardBiasMinutes`, `DaylightBiasMinutes` | `Bias`, `ActiveTimeBias` (ที่มีผลเมื่อคีย์ถูกเขียน), `StandardBias`, `DaylightBias` |
| `StandardName`, `DaylightName` | ตามที่เขียนไว้ ซึ่งมักเป็นการอ้างอิง resource (`@tzres.dll,-112`) |
| `StandardStart`, `DaylightStart` | เวลาที่เวลามาตรฐานและเวลาออมแสงเริ่มต้น ในรูปคำบรรยาย (`first Sunday of November, 02:00`) |
| `DynamicDaylightTimeDisabled`, `RealTimeIsUniversal` | การปิดใช้งานเวลาออมแสงแบบไดนามิก และนาฬิกาฮาร์ดแวร์เก็บเวลาเป็น UTC |
| `TimeZoneLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์เขตเวลา |
| `LastShutdownUTC` | `ShutdownTime`: การปิดเครื่องอย่างสมบูรณ์ครั้งล่าสุด |
| `CurrentControlSet`, `ProcessorArchitecture`, `AnsiCodePage`, `OemCodePage` | control set ที่อ่าน `PROCESSOR_ARCHITECTURE`, `ACP` และ `OEMCP` |
| `LastLoggedOnUser`, `LastLoggedOnSamUser`, `LastLoggedOnUserSID`, `LastLoggedOnDisplayName`, `LogonUiLastWrittenUTC` | ผู้ใช้ล่าสุดของหน้าจอลงชื่อเข้าใช้ และเวลาเขียนล่าสุดของคีย์ |
| `MachineSID`, `DomainSID`, `MachineAccountSID` | SID ของคอมพิวเตอร์ SID ของโดเมน (`PolPrDmS`) และ SID ของบัญชีคอมพิวเตอร์ในโดเมน (`PolMachineAccountS`) |
| `CompNameLastWrittenUTC`, `PrimaryDomainLastWrittenUTC`, `VersionLastWrittenUTC` | เวลาโดยประมาณที่คอมพิวเตอร์ถูกตั้งชื่อครั้งล่าสุด เข้าร่วมโดเมนหรือเวิร์กกรุ๊ปครั้งล่าสุด และอัปเดตครั้งล่าสุด (คีย์ของ `CurrentVersion`) |
| `PreviousVersions` | คีย์ `Source OS` แต่ละคีย์: เวอร์ชัน วันที่ติดตั้ง และวันที่ในชื่อคีย์ (JSON: หนึ่งอ็อบเจกต์ต่อรายการ) |
| `Profiles` | แต่ละโปรไฟล์: SID โฟลเดอร์ เวลาของคีย์ เวลาโหลดและยกเลิกการโหลดครั้งล่าสุด (JSON: หนึ่งอ็อบเจกต์ต่อรายการ) |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรีของไฮฟ์ทั้งสี่ |
| `Problems` | ค่าที่มีชนิดหรือขนาดไม่ถูกต้อง (เก็บไบต์ไว้) และ `SAM` กับ `SECURITY` ที่มี SID ของคอมพิวเตอร์ต่างกัน |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. อ่านเขตเวลาก่อน: จดบันทึก `UtcOffset` วันที่ของเวลาออมแสง และเขตเวลาเพิ่ง
   ถูกเปลี่ยนหรือไม่ ใช้ข้อมูลนี้แปลงบันทึกที่เป็นเวลาท้องถิ่น
2. จดบันทึกเวอร์ชัน build และวันที่ติดตั้ง ตรวจสอบ `PreviousVersions` เพื่อดูการอัปเกรด และเปรียบเทียบ
   วันที่ติดตั้งกับเวลาที่เก่าที่สุดของระบบไฟล์และ event log
3. จดบันทึก `LastShutdownUTC`: กิจกรรมหลังจากเวลานี้โดยไม่มีเวลาปิดเครื่องใหม่บ่งชี้ว่าเกิดการแครช ไฟ
   ดับ หรือเป็นการเก็บรวบรวมขณะระบบทำงาน (live collection)
4. ใช้ `MachineSid` และ `DomainSid` เพื่อแยกบัญชีภายในเครื่องกับบัญชีโดเมนใน SID ที่พบในที่อื่น (BAM,
   `ProfileList`, event log) และใช้เวลาโหลดและยกเลิกการโหลดของโปรไฟล์เพื่อดูการลงชื่อเข้าใช้ครั้งล่าสุด
5. เชื่อมโยงชื่อคอมพิวเตอร์และชื่อโดเมนกับ event log และกับ `CompNameLastWrittenUTC`
   และ `PrimaryDomainLastWrittenUTC` เพื่อดูการเปลี่ยนชื่อและการเข้าร่วมโดเมน

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-time_zone_information)
- [Microsoft: DYNAMIC_TIME_ZONE_INFORMATION](https://learn.microsoft.com/en-us/windows/win32/api/timezoneapi/ns-timezoneapi-dynamic_time_zone_information)
- [Microsoft: TIME_FIELDS](https://learn.microsoft.com/en-us/windows-hardware/drivers/ddi/wdm/ns-wdm-_time_fields)
- [Microsoft: Security identifiers](https://learn.microsoft.com/en-us/windows-server/identity/ad-ds/manage/understand-security-identifiers)
- [libyal winreg-kb: เขตเวลา](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Time-zones.md)
- [libyal winreg-kb: ข้อมูลผลิตภัณฑ์ Windows](https://github.com/libyal/winreg-kb/blob/main/docs/sources/system-keys/Windows-product-information.md)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
