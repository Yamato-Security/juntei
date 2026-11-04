# ตัวบอกความปลอดภัย (`$Secure:$SDS`) { #security-descriptors-securesds }

## `$SDS` คืออะไร { #what-sds-is }

ไฟล์และไดเรกทอรีทุกรายการบน NTFS มี **security descriptor (ตัวบอกความปลอดภัย)**: เจ้าของ กลุ่มหลัก
**DACL** (ใครทำอะไรกับมันได้บ้าง) และ **SACL** (การเข้าถึงใดที่ถูกตรวจสอบ และป้าย integrity ของมัน)
NTFS เก็บ descriptor ที่แตกต่างกันแต่ละรายการไว้เพียงครั้งเดียวในสตรีม `$SDS` ของไฟล์ `$Secure` (MFT entry 9)
และกำหนด **security ID** ให้ `$STANDARD_INFORMATION` ของไฟล์บันทึกเพียง security ID นั้น ซึ่งก็คือ
คอลัมน์ `SecurityID` ของ [`parse-ntfs mft`](mft.md) โดย `$SDS` จะแปลงค่านั้นเป็นเจ้าของและ ACL

`$SDS` แบ่งเป็นบล็อกขนาด 256 KiB แต่ละบล็อกตามด้วยสำเนาของตัวเอง (mirror) แต่ละ entry มีแฮชของ
descriptor ซึ่ง NTFS ใช้ค้นหา descriptor ที่มีอยู่แล้ว (`$SDH`) และมี
security ID (ซึ่ง `$SII` ทำดัชนีไว้)

<span class="evidence-strength">ข้อสรุปที่ดีที่สุด:</span> ไฟล์ที่ `SecurityID` ระบุ
descriptor นี้ มีเจ้าของและ ACL เหล่านั้น ณ เวลาที่ระเบียน `$MFT` ของไฟล์ถูกเขียนครั้งล่าสุด เจ้าของ
ไฟล์มักเป็นบัญชีที่สร้างไฟล์ แต่ผู้ดูแลระบบ โปรแกรมติดตั้ง หรือโปรแกรมที่
เข้าครอบครองความเป็นเจ้าของ (take ownership) สามารถเปลี่ยนแปลงได้

## ตำแหน่งและการเก็บรวบรวม { #location-and-collection }

```text
C:\$Secure:$SDS   (MFT entry 9, the $SDS data stream)
```

ให้เก็บรวบรวมด้วยเครื่องมือนิติวิทยาศาสตร์ที่อ่านข้อมูลดิบได้ การเก็บรวบรวมมักตั้งชื่อเป็น `$Secure_$SDS` เนื่องจาก `:`
ใช้ในชื่อไฟล์ไม่ได้ ให้เก็บรวบรวม `$MFT` ของวอลุ่มเดียวกันไปพร้อมกัน

## ข้อควรระวังสำหรับนักวิเคราะห์ { #analyst-cautions }

- เจ้าของคือ SID มีเพียง SID ที่รู้จักกันทั่วไป (`NT AUTHORITY\SYSTEM`, `BUILTIN\Administrators`,
  `NT SERVICE\TrustedInstaller`, …) และบัญชีในตัวของเครื่องหรือโดเมน (`-500`
  `Administrator`, …) เท่านั้นที่มีชื่อ ส่วนที่เหลือให้แปลงด้วย `SAM` ของโฮสต์
  ([บัญชี SAM](sam.md)) รายการโปรไฟล์ ([ข้อมูลระบบ](system-info.md)) หรือโดเมน
- descriptor ถูกใช้ร่วมกันโดยไฟล์ทุกไฟล์ที่มีเจ้าของและ ACL เดียวกัน และมักคงอยู่ใน `$SDS`
  หลังจากไฟล์เหล่านั้นถูกลบ descriptor ที่ไม่มีไฟล์ใน `$MFT` ไม่ได้หมายถึง
  ไฟล์ที่ถูกลบด้วยตัวมันเอง
- ACE ไม่ใช่ประวัติการเข้าถึง DACL บอกว่าใคร *สามารถ* เข้าถึงไฟล์ได้ ส่วน event log และอาร์ติแฟกต์
  อื่นบอกว่าใครเข้าถึงจริง รายการ audit ของ SACL อธิบายการกำหนดค่า ไม่ใช่สิ่งที่ถูกบันทึกไว้
- ACE แสดงตามลำดับใน ACL Juntei ไม่ได้คำนวณสิทธิ์การเข้าถึงที่มีผลจริง: รายการ
  deny การสืบทอด การเป็นสมาชิกกลุ่ม privilege และสิทธิ์โดยนัยของเจ้าของล้วนมีผล
- `DACL` และ `SACL` เขียน ACE แต่ละรายการแบบเดียวกับที่ `icacls` เขียน `SDDL` แสดงสิทธิ์ที่รองรับ โดยมี mask เป็นเลขฐานสิบหกและบัญชีเป็น SID ไม่ใช่สำเนาแบบไบต์ต่อไบต์ของ descriptor ไบนารี ข้อความเงื่อนไขที่แสดงไม่ได้จะทำให้ค่านี้ว่างเปล่าและมีการวินิจฉัยไว้

## วิเคราะห์ด้วย Juntei { #parse-with-juntei }

```text
Usage: juntei parse-ntfs secure-sds <INPUT> [OPTIONS]

Input:
  -d, --directory <DIR>  Recursively find $SDS streams (with $SDS in their name, or starting with an $SDS entry)
  -f, --file <FILE>      Parse one $Secure:$SDS stream (often exported as $Secure_$SDS)

Output:
  -o, --output <FILE>       Save every security descriptor
  -t, --output-type <TYPE>  csv, json, or jsonl
```

```bash
juntei parse-ntfs secure-sds -f '$Secure_$SDS'
juntei parse-ntfs secure-sds -d evidence/ -o sds.csv
juntei parse-ntfs secure-sds -f '$Secure_$SDS' -t jsonl
```

ส่วนสรุปนับจำนวน descriptor และแสดงรายการเจ้าของ พร้อมจำนวน descriptor ที่แต่ละรายเป็นเจ้าของ
descriptor ที่มี NULL DACL (ไม่มีการจำกัดใด ๆ เลย) และความเสียหายใด ๆ

## ฟิลด์เอาต์พุต CSV { #csv-output-fields }

| ฟิลด์ | การตีความ |
|---|---|
| `SourcePath` | ไฟล์อินพุต |
| `SecurityID` | security ID: ใช้เชื่อม (join) กับ `SecurityID` ของ [`parse-ntfs mft`](mft.md) สำหรับวอลุ่มเดียวกัน |
| `Hash` | แฮชที่บันทึกไว้พร้อมกับ descriptor |
| `EntryOffset` | ตำแหน่งของ entry ในสตรีม |
| `FromMirror` | `True` สำหรับสำเนาที่อ่านจากบล็อก mirror: สำเนาที่แตกต่างจาก entry ของตน (แสดงต่อจาก entry นั้น) หรือสำเนาของ entry ที่เสียหาย |
| `Size` | ขนาดของ entry รวมส่วนหัว 20 ไบต์ |
| `Owner`, `OwnerName` | SID ของเจ้าของ และชื่อหากเป็น SID ที่รู้จักกันทั่วไป |
| `Group`, `GroupName` | SID ของกลุ่มหลัก และชื่อหากเป็น SID ที่รู้จักกันทั่วไป Windows ไม่ได้ใช้ค่านี้ในการตรวจสอบสิทธิ์การเข้าถึง |
| `Control` | แฟล็กควบคุมของ descriptor: `SE_DACL_PRESENT`, `SE_DACL_PROTECTED` (ปิดกั้นการสืบทอด), `SE_DACL_AUTO_INHERITED`, … เชื่อมด้วย `|` |
| `DACL-ACE-Count`, `DACL` | รายการของ DACL คั่นด้วย `; ` ว่างเปล่าหากไม่มี DACL และเป็น `NULL DACL: no restriction` หาก descriptor ให้สิทธิ์ทุกอย่างแก่ทุกคน |
| `SACL-ACE-Count`, `SACL` | รายการของ SACL: รายการ audit ป้าย mandatory integrity ป้าย process trust และ resource attribute |
| `SDDL` | descriptor ทั้งหมดในรูปแบบ SDDL ว่างเปล่าเฉพาะเมื่อมีบางส่วนที่ถอดรหัสไม่ได้ |
| `Problems` | แฮชที่ไม่ตรงกับ descriptor ส่วนที่ถอดรหัสไม่ได้ และ ACE ที่ไม่รู้จักชนิด |

รายการแต่ละรายการของ `DACL` และ `SACL` เขียนแบบเดียวกับที่ `icacls` เขียน: บัญชี ชนิดหาก
ไม่ใช่รายการ allow แฟล็กการสืบทอด และสิทธิ์

| ส่วน | ความหมาย |
|---|---|
| บัญชี | ชื่อที่รู้จักกันทั่วไป หรือ SID |
| `(DENY)`, `(AUDIT)`, `(ACCESS_ALLOWED_CALLBACK)`, … | ชนิดของ ACE เมื่อไม่ใช่รายการ allow ป้าย mandatory จะไม่แสดงชนิด |
| `(I)` | สืบทอดมาจากไดเรกทอรีแม่ |
| `(OI)`, `(CI)`, `(IO)`, `(NP)` | สืบทอดไปยังไฟล์ (object inherit) ไปยังไดเรกทอรี (container inherit) ไปยังเฉพาะไฟล์/ไดเรกทอรีเหล่านั้นแต่ไม่ใช่ออบเจ็กต์นี้ (inherit only) และลงไปเพียงหนึ่งระดับ (no propagate) |
| `(SA)`, `(FA)` | รายการ audit สำหรับการเข้าถึงที่สำเร็จหรือล้มเหลว |
| `(F)`, `(M)`, `(RX)`, `(R)`, `(W)` | ควบคุมเต็มรูปแบบ แก้ไข อ่านและเรียกใช้ อ่าน เขียน |
| `(RD,WD,AD,…)` | สิทธิ์เฉพาะในกรณีอื่น: อ่านข้อมูล เขียนข้อมูล ต่อท้ายข้อมูล อ่านและเขียน extended attribute (`REA`, `WEA`) เรียกใช้ (`X`) ลบ child (`DC`) อ่านและเขียนแอตทริบิวต์ (`RA`, `WA`) ลบ (`DE`) read control (`RC`) write DAC (`WDAC`) write owner (`WO`) synchronize (`S`) และสิทธิ์ทั่วไป `GA`, `GR`, `GW`, `GE` ส่วน `N` คือไม่มีสิทธิ์เข้าถึง |
| `(NW)`, `(NR)`, `(NX)` | นโยบายของป้าย mandatory: no write up, no read up, no execute up |
| `if (…)` | เงื่อนไขของ callback ACE เช่น `(WIN://SYSAPPID Contains "…")`: รายการนี้มีผลเฉพาะกับโพรเซสที่เงื่อนไขเป็นจริง Windows ใช้สิ่งนี้เพื่อให้สิทธิ์เข้าถึงแก่แพ็กเกจ app container |

ป้าย process trust และ resource attribute แสดง mask เป็นเลขฐานสิบหก และ resource
attribute แสดง claim ของตน (`("IMAGELOAD",TU,0x0,1)`)

JSON/JSONL มีฟิลด์และชื่อเหมือนกัน โดยตัวเลขเป็นตัวเลข ค่าว่างเป็น `null` และ
`Problems` เป็นรายการ

## กลยุทธ์การตรวจสอบ { #investigation-strategy }

1. วิเคราะห์ `$MFT` ของวอลุ่มเดียวกัน และเชื่อม `SecurityID` ของมันกับของเอาต์พุตนี้: ไฟล์แต่ละไฟล์จะได้
   เจ้าของและ ACL ของตน
2. พิจารณาเจ้าของที่เป็นบัญชีผู้ใช้ (`S-1-5-21-…`): ไฟล์ใดที่บัญชีที่น่าสงสัยเป็น
   เจ้าของ และบัญชีใดเป็นเจ้าของไฟล์ในไดเรกทอรีระบบ
3. มองหา ACL ที่อนุญาตกว้างเกินไปบนไดเรกทอรีที่โปรแกรมทำงานหรือโหลดข้อมูล: `Everyone`,
   `Authenticated Users` หรือ `User` ที่มี `(F)`, `(M)` หรือ `(W)` และ NULL DACL
4. descriptor ที่ถูกใช้โดยไฟล์เพียงไฟล์เดียว หรือโดยกลุ่มไฟล์ที่ผิดปกติ อาจแสดงไฟล์ที่ถูก
   สร้างหรือเปลี่ยนแปลงนอกเหนือจากการติดตั้งตามปกติ
5. ยืนยันการเปลี่ยนแปลงความเป็นเจ้าของและสิทธิ์ด้วยรายการใน Security event log (4670, 4663 เมื่อมี
   SACL) และระเบียน `SECURITY_CHANGE` ของ[บันทึก USN](usn-journal.md)

## รายละเอียดเพิ่มเติม { #further-details }

- [Microsoft: Security descriptors](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptors)
- [Microsoft Open Specifications: MS-DTYP](https://learn.microsoft.com/en-us/openspecs/windows_protocols/ms-dtyp/) (security descriptor, ACE และนิพจน์เงื่อนไข)
- [Microsoft: Security Descriptor String Format (SDDL)](https://learn.microsoft.com/en-us/windows/win32/secauthz/security-descriptor-string-format)
- [Microsoft: icacls](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/icacls)
- [libfsntfs: เอกสารรูปแบบ NTFS](https://github.com/libyal/libfsntfs/blob/main/documentation/New%20Technologies%20File%20System%20(NTFS).asciidoc) (`$Secure`)
- [ys-ntfs-parser](https://github.com/Yamato-Security/ys-ntfs-parser) (crate `secure` ของโปรเจกต์)

เมื่อส่งเอาต์พุตแบบมีโครงสร้างไปยัง stdout (`-t` โดยไม่มี `-o`) คำเตือนการกู้คืนระดับสตรีมจะถูกส่งไปยัง stderr ทำให้ pipe ยังคงได้ CSV/JSON/JSONL ที่อ่านได้ โดยไม่ปกปิดบริเวณที่เสียหายหรือที่ถูกกู้คืน
