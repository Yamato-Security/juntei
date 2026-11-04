# การกำหนดค่าบูต (BCD) { #boot-configuration-data-bcd }

## BCD store คืออะไร { #what-the-bcd-store-is }

Windows Vista และรุ่นใหม่กว่าจัดเก็บการกำหนดค่าบูตไว้ในไฮฟ์รีจิสทรีที่เรียกว่า BCD store ได้แก่
`\Boot\BCD` บนระบบ BIOS และ `\EFI\Microsoft\Boot\BCD` บน EFI system partition ของระบบ
UEFI โดย `bcdedit` และตัวติดตั้ง Windows เป็นผู้แก้ไข ส่วน boot manager จะอ่านค่านี้ทุกครั้งที่บูต

store ประกอบด้วยอ็อบเจกต์ ซึ่งแต่ละอ็อบเจกต์ตั้งชื่อตาม GUID ได้แก่

- **Windows boot manager** (`{bootmgr}`) ซึ่งมีรายการเริ่มต้น ลำดับการแสดงผล และ
  ระยะเวลารอของเมนู บนระบบ UEFI ยังมี **firmware boot manager** (`{fwbootmgr}`) ที่เก็บลำดับการบูตของ
  เฟิร์มแวร์ด้วย
- **Windows boot loader** หนึ่งรายการต่อการติดตั้ง Windows แต่ละชุด (รวมถึงสภาพแวดล้อมการกู้คืน)
  **resume application** (ไฮเบอร์เนต) และ **memory tester**
- อ็อบเจกต์ **inherited settings** (`{globalsettings}`, `{bootloadersettings}`, …) ซึ่งอ็อบเจกต์อื่น
  สืบทอดการตั้งค่าไป
- อ็อบเจกต์ **device** (ตัวเลือก ramdisk ของสภาพแวดล้อมการกู้คืน)

แต่ละอ็อบเจกต์มี element ได้แก่ คำอธิบาย อุปกรณ์และพาธ และการตั้งค่าต่าง ๆ เช่น
`recoveryenabled`, `bootstatuspolicy`, `testsigning`, `nointegritychecks` และ `safeboot`

การโจมตีสองประเภทที่เปลี่ยนแปลงค่าเหล่านี้ ได้แก่

- **แรนซัมแวร์** จะปิดการกู้คืนเพื่อไม่ให้ระบบซ่อมแซมได้:
  `bcdedit /set {default} recoveryenabled No` และ
  `bcdedit /set {default} bootstatuspolicy ignoreallfailures` บางตระกูลจะตั้งค่า `safeboot` เพื่อ
  ให้ Windows รีสตาร์ทในเซฟโหมด ซึ่งผลิตภัณฑ์ความปลอดภัยไม่ทำงาน
- **รูทคิทและไดรเวอร์ที่ไม่ได้ลงนาม** อาจใช้ `testsigning on` หรือ `nointegritychecks on` เพื่อลดความเข้มงวดของการตรวจสอบขณะบูต การตั้งค่าเหล่านี้เพียงอย่างเดียวไม่ได้พิสูจน์ว่ามีรูทคิททำงาน

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ณ เวลาที่เก็บรวบรวม รายการบูตนี้
มีการตั้งค่านี้ เวลาเขียนล่าสุดของคีย์ของ element คือเวลาที่การตั้งค่านั้นถูกเขียนครั้งล่าสุด

## ตำแหน่ง { #location }

```text
\Boot\BCD                              BIOS systems (the system partition)
\EFI\Microsoft\Boot\BCD                UEFI systems (the EFI system partition)
C:\Windows\System32\config\BCD-Template   the template new stores are made from
Objects\{GUID}\Description             Type
Objects\{GUID}\Elements\<type>         Element
```

ชนิดของ element เป็นเลขฐานสิบหก 8 หลัก ประกอบด้วยคลาส (`1` ทุกแอปพลิเคชัน, `2` แอปพลิเคชันชนิดหนึ่ง,
`3` อุปกรณ์) รูปแบบ (`1` อุปกรณ์, `2` สตริง, `3` อ็อบเจกต์, `4` รายการอ็อบเจกต์, `5`
จำนวนเต็ม, `6` บูลีน, `7` รายการจำนวนเต็ม) และหมายเลข คำสั่งนี้ตั้งชื่อ element ตามที่
Microsoft ระบุไว้ในเอกสาร (`BcdLibraryBoolean_AutoRecoveryEnabled`)

เครื่องมือเก็บรวบรวมข้อมูลมักไม่คัดลอก store นี้ เนื่องจาก EFI system partition ไม่ได้ถูกเมานต์ และ `\Boot`
ถูกซ่อนอยู่ ให้เก็บรวบรวมโดยระบุอย่างชัดเจน หรือเมานต์พาร์ทิชันนั้น `-d` จะอ่าน `BCD-Template` ด้วย
(`HiveType` ระบุว่าเป็นชนิดใด) เทมเพลตคือจุดเริ่มต้นที่ Windows ใช้ ไม่ใช่ store ที่ใช้บูตจริง

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- **การตั้งค่ามีการสืบทอด** boot loader จะสืบทอดค่าจากอ็อบเจกต์ใน
  `BcdLibraryObjectList_InheritedObjects` ของตน (โดยปกติคือ `{bootloadersettings}` ซึ่งสืบทอดจาก
  `{globalsettings}`) `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks`, `SafeBoot` และ
  `BootStatusPolicy` เป็นค่าที่จัดเก็บไว้ซึ่งได้รับการ resolve แล้ว (ค่าของตนเอง หรือค่าที่สืบทอดมา) และ `Inherited` ระบุว่าค่าใด
  มาจากอ็อบเจกต์อื่น การติดตามการสืบทอดทำได้สูงสุดแปดระดับ โดยมีการจำกัดกรณีวนซ้ำ อ็อบเจกต์ที่ถูกอ้างอิงอาจไม่มีอยู่ใน store ที่ไม่สมบูรณ์ ฟิลด์ที่ว่างไม่ได้ยืนยันค่าเริ่มต้นที่มีผลจริงของ Windows นอกจากนี้เฟิร์มแวร์และ Secure Boot ก็อาจทำให้ตัวเลือกที่จัดเก็บไว้ไม่มีผลได้
- **ไม่ได้ตั้งค่าไม่ได้แปลว่าปิด** `RecoveryEnabled` ที่ว่างหมายความว่าไม่มี element นั้นอยู่ และจะใช้ค่าเริ่มต้นของ
  Windows
- **เวลาเป็นรายบุคคลของแต่ละ element** เวลาของคีย์ของแต่ละ element (`key_last_written` ใน `Elements`) คือเวลาที่
  element นั้นถูกเขียนครั้งล่าสุด Windows จะเขียน element บางรายการใหม่เมื่ออัปเดตและอัปเกรด ให้เปรียบเทียบกับเวลาของ
  element อื่น ๆ
- **อุปกรณ์ถูกถอดรหัสเพียงบางส่วน** พาร์ทิชัน GPT จะแสดงเป็น GUID ของพาร์ทิชันและดิสก์
  ramdisk จะแสดงเป็นอ็อบเจกต์ตัวเลือกของมัน ส่วนอุปกรณ์อื่นจะเก็บไว้เพียงไบต์ดิบ (`data_raw`)
- **element ที่ไม่มีเอกสาร** (`0x15000065`, คลาส `0x4…`) ไม่มีชื่อและจะเก็บไว้เป็นไบต์

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-registry bcd <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find BCD hives and BCD-Template
  -f, --file <FILE>      Parse one BCD hive
      --log1/--log2      Supply transaction logs explicitly
      --no-logs          Do not replay logs

Output:
  -o, --output <FILE>       Save records
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-registry bcd -f BCD -o bcd.csv
juntei parse-registry bcd -d evidence/ -t jsonl \
  | jq 'select(.TestSigning == true or .NoIntegrityChecks == true or .RecoveryEnabled == false)'
```

ข้อมูลสรุปจะแสดงรายการเริ่มต้นของแต่ละ store และการตั้งค่า test signing, การตรวจสอบความสมบูรณ์, การกู้คืน,
boot status policy (ค่าใดก็ตามที่เป็น `Ignore…`) และเซฟโหมดทุกรายการ พร้อมเวลาที่ตั้งค่าหรืออ็อบเจกต์
ที่สืบทอดค่ามา

## ฟิลด์เอาต์พุต { #output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `Host`, `SourcePath`, `HiveType` | โฮสต์ ไฮฟ์ และ `BCD` หรือ `BCD-Template` |
| `ObjectGUID`, `ObjectType`, `ObjectTypeName` | GUID ของอ็อบเจกต์ ชนิด (`0x10200003`) และสิ่งที่อ็อบเจกต์นั้นเป็น (`Windows boot loader`, `inherited settings for Windows boot loader`, …) |
| `Description`, `Path`, `Device` | คำอธิบาย พาธของแอปพลิเคชัน และอุปกรณ์ (พาร์ทิชันและดิสก์ GPT หรืออ็อบเจกต์ตัวเลือกของ ramdisk) |
| `IsDefault`, `DisplayOrder` | Windows boot manager บูตรายการนี้เป็นค่าเริ่มต้นหรือไม่ และตำแหน่งในเมนูบูต |
| `RecoveryEnabled`, `TestSigning`, `NoIntegrityChecks` | `recoveryenabled`, `testsigning`, `nointegritychecks` ตามที่ resolve จาก store |
| `SafeBoot`, `BootStatusPolicy` | `safeboot` (`Minimal`, `Network`, `DsRepair`) และ `bootstatuspolicy` (`IgnoreAllFailures`, …) พร้อมค่าตัวเลข |
| `Inherited` | ค่าใดในกลุ่มนี้มาจากอ็อบเจกต์อื่น และมาจากอ็อบเจกต์ใด |
| `Elements` | element ทุกรายการ: ชนิด ชื่อ ค่า ไบต์ (สำหรับอุปกรณ์ ชนิดที่ไม่มีเอกสาร และข้อมูลที่ถอดรหัสไม่ได้) และเวลาของคีย์ |
| `KeyLastWrittenUTC` | เวลาเขียนล่าสุดของคีย์ของอ็อบเจกต์ |
| `TransactionLogStatus`, `LogsApplied` | บริบทการกู้คืนรีจิสทรี |
| `Problems` | อ็อบเจกต์ที่ไม่มีชนิด และ element ที่ข้อมูลไม่ตรงกับรูปแบบ |

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. ตรวจสอบรายการเริ่มต้นและ boot loader ทุกรายการว่ามี `TestSigning`, `NoIntegrityChecks`,
   `RecoveryEnabled = false`, `BootStatusPolicy` ที่เป็น `Ignore…` และ `SafeBoot` หรือไม่
2. ใช้เวลาของ element เพื่อระบุช่วงเวลาที่เปลี่ยนแปลง และเชื่อมโยงกับ `bcdedit.exe` ใน Prefetch,
   Amcache, เหตุการณ์การสร้างโพรเซส (4688, Sysmon 1) และ System event log
3. มองหารายการบูตหรืออุปกรณ์ที่ไม่ควรมีอยู่ (พาธ loader อื่น หรือ ramdisk) และ
   ค่าเริ่มต้นหรือลำดับการแสดงผลที่ถูกเปลี่ยน
4. เปรียบเทียบ store กับ `BCD-Template` และกับระบบอื่นที่มี build เดียวกัน

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: BCD boot options reference](https://learn.microsoft.com/en-us/windows-hardware/drivers/devtest/bcd-boot-options-reference)
- [Microsoft: BcdLibraryElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdlibraryelementtypes)
- [Microsoft: BcdOSLoaderElementTypes](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdosloaderelementtypes)
- [Microsoft: BcdObject (ชนิดอ็อบเจกต์)](https://learn.microsoft.com/en-us/previous-versions/windows/desktop/bcd/bcdobject)
- [ys-registry-parser](https://github.com/Yamato-Security/ys-registry-parser)
